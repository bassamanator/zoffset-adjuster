#[cfg(test)]
mod tests {

    #[test]
    fn test_revert_at_layer_height_4_layers() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            4,
        );
        assert_eq!(params.revert_z_offset_at_height(), 0.86);
    }

    #[test]
    fn test_revert_at_layer_height_6_layers() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            6,
        );
        assert_eq!(params.revert_z_offset_at_height(), 1.26);
    }

    #[test]
    fn test_revert_at_layer_height_2_layers() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            2,
        );
        assert_eq!(params.revert_z_offset_at_height(), 0.46);
    }
}
