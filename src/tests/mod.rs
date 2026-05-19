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

    #[test]
    fn test_revert_at_layer_height_1_000_000_layers() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            1_000_000,
        );
        let ans = ((1_000_000 - 1) as f32) * params.layer_height + params.first_layer_height;
        assert_eq!(params.revert_z_offset_at_height(), ans);
    }

    #[test]
    fn test_adjust_z_offset_code() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            2,
        );
        assert_eq!(
            params.adjust_z_offset_code(),
            "SET_GCODE_OFFSET Z_ADJUST=-0.015 MOVE=1".to_owned()
        );
    }

    #[test]
    fn test_adjust_z_offset_code2() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            0.029,
            0.26,
            0.2,
            2,
        );
        assert_eq!(
            params.adjust_z_offset_code(),
            "SET_GCODE_OFFSET Z_ADJUST=+0.029 MOVE=1".to_owned()
        );
    }

    #[test]
    fn test_revert_z_offset_code() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            -0.015,
            0.26,
            0.2,
            2,
        );

        println!("params: {:#?}", params);
        println!(
            "params.revert_z_offset_code(): {:?}",
            params.revert_z_offset_code()
        );
        assert_eq!(
            params.revert_z_offset_code(),
            "SET_GCODE_OFFSET Z_ADJUST=+0.015 MOVE=1".to_owned()
        );
    }

    #[test]
    fn test_revert_z_offset_code2() {
        let params = crate::helpers::ZOffsetAdjustmentParams::new(
            "test.gcode".to_string(),
            0.123,
            0.26,
            0.2,
            2,
        );

        println!("params: {:#?}", params);
        println!(
            "params.revert_z_offset_code(): {:?}",
            params.revert_z_offset_code()
        );
        assert_eq!(
            params.revert_z_offset_code(),
            "SET_GCODE_OFFSET Z_ADJUST=-0.123 MOVE=1".to_owned()
        );
    }
}
