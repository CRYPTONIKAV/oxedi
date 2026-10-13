//! Masking of the columns the spec flags as patient data, in the rows the
//! projector appends and in the datums of the diagnostics it hands out.
//!
//! A row is masked just before it is appended, so every column has its final
//! value whatever order the segments came in. Text of a `name` or `id` column
//! becomes its keyed token; an empty text stays empty and a null stays null.
//! A `date` column follows the mask's date mode: the year keeps January 1 of
//! the same year, a shift adds the days the row's seed gives, and keep leaves
//! it. The seed is the unmasked value of the table's seed column, or, for a
//! table that declares none, of the nearest table above it that declares one
//! (its row is still open, so still unmasked). A row with no seed value, or
//! whose shift would leave the date range, gets the year instead.
//!
//! A diagnostic's datum is masked when the place it names (segment id,
//! element, component) is one a flagged column can read: a `name` or `id`
//! datum becomes its token, a `date` datum that is a valid date becomes the
//! masked date written `CCYYMMDD` (the year, as no row is at hand to seed a
//! shift), and one that is not becomes its token. An element read whole
//! covers its components, and a finding about a whole element covers the
//! components read from it. Places are matched by segment id alone, so the
//! same element of a segment in another loop, or with another qualifier, is
//! masked too. The segment id is the one the rule names, or the segment being
//! fed when the diagnostic is about it; a balancing finding names an earlier
//! segment, but its datum is an `R` amount, which no flagged column reads.
//! Findings another validator raised about a document are matched the same
//! way, by the segment their index names in it, and an external finding whose
//! datum is masked has every occurrence of the original datum in its message
//! replaced by the masked one.

use crate::column::{civil_from_days, parse_dt};
use crate::diagnostic::{Diagnostic, Rule};
use crate::document::Document;
use crate::mask::{DateMode, Mask, Phi};
use crate::segment::Segment;
use crate::spec::Spec;

use super::plan::Plans;
use super::{Projector, Row, Slot};

/// The places flagged columns read, and the mask that rewrites the datums
/// found there.
#[derive(Debug, Clone)]
pub(super) struct DatumMask<'s> {
    mask: &'s Mask,
    places: Vec<(&'s [u8], usize, Option<usize>, Phi)>,
}

impl<'s> DatumMask<'s> {
    /// The places every flagged column of `spec` reads, as the projector's
    /// plans resolve them.
    pub(super) fn new(spec: &'s Spec, plans: &Plans<'_>, mask: &'s Mask) -> Self {
        let mut places = Vec::new();
        for table in spec.tables() {
            for (index, (_, source)) in table.columns.iter().enumerate() {
                let Some(kind) = table.phi(index) else {
                    continue;
                };
                for (id, element, component) in plans.positions(table, source) {
                    places.push((id, element, component, kind));
                }
            }
        }
        Self { mask, places }
    }

    /// Masks the datum of each diagnostic about a flagged place; `current`
    /// is the segment being fed, if any.
    pub(super) fn apply(&self, diagnostics: &mut [Diagnostic], current: Option<&Segment<'_>>) {
        for diagnostic in diagnostics {
            if diagnostic.datum.is_empty() {
                continue;
            }
            let Some(element) = diagnostic.element else {
                continue;
            };
            let id = match diagnostic.rule.segment_id() {
                Some(id) => id,
                None => match current {
                    Some(segment) if diagnostic.segment == Some(segment.index) => segment.id,
                    _ => continue,
                },
            };
            let component = diagnostic.component;
            let found = self.places.iter().find(|(at, position, part, _)| {
                *at == id
                    && *position == element
                    && (part.is_none() || component.is_none() || *part == component)
            });
            if let Some(&(.., kind)) = found
                && let Some(masked) = self.datum(kind, &diagnostic.datum)
            {
                let original = std::mem::replace(&mut diagnostic.datum, masked);
                if let Rule::External { message, .. } = &mut diagnostic.rule {
                    let original = String::from_utf8_lossy(&original);
                    if !original.is_empty() {
                        let masked = String::from_utf8_lossy(&diagnostic.datum);
                        *message = message.replace(original.as_ref(), masked.as_ref());
                    }
                }
            }
        }
    }

    /// The masked form of a datum, or `None` when it stays as written.
    fn datum(&self, kind: Phi, datum: &[u8]) -> Option<Vec<u8>> {
        match kind {
            Phi::Name | Phi::Id => Some(self.mask.text(kind, datum).to_vec()),
            Phi::Date if self.mask.dates() == DateMode::Keep => None,
            Phi::Date => Some(match parse_dt(datum) {
                Some(days) => {
                    let (year, month, day) = civil_from_days(Mask::year_start(days));
                    format!("{year:04}{month:02}{day:02}").into_bytes()
                }
                None => self.mask.text(Phi::Date, datum).to_vec(),
            }),
        }
    }
}

impl<'s> Projector<'s> {
    /// The same projector, masking the columns the spec flags as patient
    /// data in every row it appends and the datums of its diagnostics read
    /// from them. With dates shifted, a table left out by
    /// [`Projector::with_tables`] that declares a seed column still reads
    /// its rows, so the tables below it shift as in a full projection.
    pub fn with_mask(mut self, mask: &'s Mask) -> Self {
        let spec = self.spec;
        for (state, def) in self.tables.iter_mut().zip(spec.tables()) {
            state.masked = (0..def.columns.len()).any(|column| def.phi(column).is_some());
            if mask.dates() == DateMode::Shift && def.phi_seed().is_some() {
                state.collect = true;
            }
        }
        self.datums = Some(DatumMask::new(spec, &self.plans, mask));
        self.mask = Some(mask);
        self
    }

    /// Masks the datums of diagnostics another consumer raised, as this
    /// projector masks its own; `current` is the segment being fed, if any.
    pub(crate) fn mask_datums(
        &self,
        diagnostics: &mut [Diagnostic],
        current: Option<&Segment<'_>>,
    ) {
        if let Some(datums) = &self.datums {
            datums.apply(diagnostics, current);
        }
    }

    /// Masks the datums of diagnostics another validator raised about
    /// `document`, as this projector masks its own: each is matched by the
    /// segment its index names. An external finding whose datum is masked
    /// also has every occurrence of the original datum in its message
    /// replaced by the masked one. Without a mask it does nothing.
    pub fn mask_found(&self, diagnostics: &mut [Diagnostic], document: &Document<'_>) {
        let Some(datums) = &self.datums else {
            return;
        };
        for diagnostic in diagnostics {
            let current = diagnostic.segment.and_then(|index| document.segment(index));
            datums.apply(std::slice::from_mut(diagnostic), current.as_ref());
        }
    }

    /// Masks the flagged columns of `row`, a row of table `index` about to
    /// be appended.
    pub(super) fn mask_row(&self, index: usize, row: &mut Row) {
        let Some(mask) = self.mask else {
            return;
        };
        let def = &self.spec.tables()[index];
        let shift = match mask.dates() {
            DateMode::Shift => self.seed(index, row).map(|seed| mask.shift_days(seed)),
            DateMode::Year | DateMode::Keep => None,
        };
        let Row { cells, bytes, .. } = row;
        for (column, slot) in cells.iter_mut().enumerate() {
            match (def.phi(column), *slot) {
                (Some(kind @ (Phi::Name | Phi::Id)), Slot::Bytes(start, end)) if end > start => {
                    let token = mask.text(kind, bytes.get(start..end).unwrap_or_default());
                    let from = bytes.len();
                    bytes.extend_from_slice(&token);
                    *slot = Slot::Bytes(from, bytes.len());
                }
                (Some(Phi::Date), Slot::Date(days)) => {
                    *slot = Slot::Date(masked_date(mask.dates(), days, shift));
                }
                _ => {}
            }
        }
    }

    /// The unmasked seed of `row`, a row of table `index`: its own seed
    /// column, or the open row of the nearest table above that declares one.
    fn seed<'r>(&'r self, index: usize, row: &'r Row) -> Option<&'r [u8]> {
        let tables = self.spec.tables();
        let def = &tables[index];
        let (source, column) = match def.phi_seed() {
            Some(column) => (row, column),
            None => {
                let above = def
                    .ancestors
                    .iter()
                    .rev()
                    .copied()
                    .find(|&above| tables[above].phi_seed().is_some())?;
                let state = &self.tables[above];
                (state.open.then_some(&state.row)?, tables[above].phi_seed()?)
            }
        };
        match source.cells.get(column)? {
            Slot::Bytes(start, end) if end > start => source.bytes.get(*start..*end),
            _ => None,
        }
    }
}

/// A date of a flagged column under `dates`, with the row's `shift` in days
/// when one could be seeded.
fn masked_date(dates: DateMode, days: i32, shift: Option<i32>) -> i32 {
    match dates {
        DateMode::Keep => days,
        DateMode::Year => Mask::year_start(days),
        DateMode::Shift => shift
            .and_then(|shift| days.checked_add(shift))
            .unwrap_or_else(|| Mask::year_start(days)),
    }
}
