use oxedi_core::DateMode;

use super::{constant, macro_parameter, mask_of};
use crate::error::ReadError;

#[test]
fn a_value_is_kept() {
    assert_eq!(
        constant("table_name", "claims".to_owned()).ok(),
        Some("claims".to_owned())
    );
}

#[test]
fn a_macro_parameter_is_refused_with_its_name() {
    let error = constant("version", "0_macro_parameterspinned.v".to_owned());
    assert!(matches!(
        error,
        Err(ReadError::MacroParameter { name: "version", ref parameter, ref value })
            if parameter.as_deref() == Some("v") && value == "0_macro_parameterspinned.v"
    ));
}

#[test]
fn the_parameter_is_the_part_after_the_one_dot() {
    assert_eq!(
        macro_parameter("0_macro_parametersm.x"),
        Some(Some("x".to_owned()))
    );
}

#[test]
fn more_dots_leave_the_parameter_unnamed() {
    assert_eq!(macro_parameter("0_macro_parametersm.a.b"), Some(None));
    let error = constant("table_name", "0_macro_parametersmy.macro.tbl".to_owned());
    assert!(matches!(
        error,
        Err(ReadError::MacroParameter {
            parameter: None,
            ..
        })
    ));
}

#[test]
fn a_value_is_not_a_macro_parameter() {
    assert_eq!(macro_parameter("claims"), None);
}

#[test]
fn the_prefix_without_a_qualified_name_is_a_value() {
    assert_eq!(
        constant("table_name", "0_macro_parameters".to_owned()).ok(),
        Some("0_macro_parameters".to_owned())
    );
}

#[test]
fn no_key_and_no_mode_is_no_mask() {
    assert!(matches!(mask_of(None, None), Ok(None)));
}

#[test]
fn a_key_alone_masks_dates_to_the_year() {
    let mask = mask_of(Some("oxedi-test-key-0".to_owned()), None)
        .ok()
        .flatten();
    assert_eq!(mask.map(|mask| mask.dates()), Some(DateMode::Year));
}

#[test]
fn a_key_with_a_mode_uses_the_mode() {
    let mask = mask_of(
        Some("oxedi-test-key-0".to_owned()),
        Some("shift".to_owned()),
    )
    .ok()
    .flatten();
    assert_eq!(mask.map(|mask| mask.dates()), Some(DateMode::Shift));
}

#[test]
fn a_mode_without_a_key_is_refused() {
    assert!(matches!(
        mask_of(None, Some("year".to_owned())),
        Err(ReadError::MaskDatesWithoutKey { ref dates }) if dates == "year"
    ));
}

#[test]
fn a_short_key_is_refused_naming_its_parameter() {
    assert!(matches!(
        mask_of(Some("short".to_owned()), None),
        Err(ReadError::Mask {
            name: "mask_key",
            ..
        })
    ));
}

#[test]
fn an_unknown_mode_is_refused_naming_its_parameter() {
    assert!(matches!(
        mask_of(
            Some("oxedi-test-key-0".to_owned()),
            Some("month".to_owned())
        ),
        Err(ReadError::Mask {
            name: "mask_dates",
            ..
        })
    ));
}
