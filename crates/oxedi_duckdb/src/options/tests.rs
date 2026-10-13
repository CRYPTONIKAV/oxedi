use super::constant;
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
            if parameter == "v" && value == "0_macro_parameterspinned.v"
    ));
}

#[test]
fn the_parameter_is_the_part_after_the_last_dot() {
    let error = constant("table_name", "0_macro_parametersmy.macro.tbl".to_owned());
    assert!(matches!(
        error,
        Err(ReadError::MacroParameter { ref parameter, .. }) if parameter == "tbl"
    ));
}

#[test]
fn the_prefix_without_a_qualified_name_is_a_value() {
    assert_eq!(
        constant("table_name", "0_macro_parameters".to_owned()).ok(),
        Some("0_macro_parameters".to_owned())
    );
}
