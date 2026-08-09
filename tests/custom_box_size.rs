use ui_layout::{BoxSizing, Length, LengthOrAuto, Spacing, Style};

fn resolve(
    style: &Style,
    intrinsic_width: f32,
    intrinsic_height: f32,
    aspect_ratio: Option<f32>,
) -> (f32, f32) {
    ui_layout::resolve_custom_box_size(
        style,
        intrinsic_width,
        intrinsic_height,
        aspect_ratio,
        Some(400.0),
        Some(300.0),
        800.0,
        600.0,
    )
}

#[test]
fn auto_sizes_use_intrinsic() {
    let style = Style::default();
    assert_eq!(resolve(&style, 200.0, 100.0, None), (200.0, 100.0));
}

#[test]
fn explicit_size_overrides_intrinsic() {
    let style = Style {
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, None), (50.0, 40.0));
}

#[test]
fn css_aspect_ratio_derives_height() {
    let style = Style {
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, Some(2.0)), (100.0, 50.0));
}

#[test]
fn css_aspect_ratio_derives_width() {
    let style = Style {
        size: ui_layout::SizeStyle {
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, Some(2.0)), (80.0, 40.0));
}

#[test]
fn intrinsic_ratio_fallback_derives_other_axis() {
    let style = Style {
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, Some(2.0)), (100.0, 50.0));
}

#[test]
fn border_box_subtracts_padding_border() {
    let style = Style {
        box_sizing: BoxSizing::BorderBox,
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Px(120.0)),
            height: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(5.0),
            padding_right: Length::Px(5.0),
            border_top: Length::Px(2.0),
            border_bottom: Length::Px(2.0),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, None), (110.0, 56.0));
}

#[test]
fn min_max_constraints_applied() {
    let style = Style {
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            min_width: LengthOrAuto::Length(Length::Px(80.0)),
            max_width: LengthOrAuto::Length(Length::Px(90.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, None), (80.0, 50.0));

    let style = Style {
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Px(95.0)),
            min_width: LengthOrAuto::Length(Length::Px(80.0)),
            max_width: LengthOrAuto::Length(Length::Px(90.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, None), (90.0, 100.0));
}

#[test]
fn max_width_clamp_preserves_aspect_ratio_when_both_auto() {
    let style = Style {
        size: ui_layout::SizeStyle {
            max_width: LengthOrAuto::Length(Length::Percent(100.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    let (w, h) = resolve(&style, 1183.0, 683.0, Some(1183.0 / 683.0));
    assert!((w - 400.0).abs() < 0.01, "width should clamp to 400");
    assert!(
        (h - 400.0 / (1183.0 / 683.0)).abs() < 0.01,
        "height should derive from the clamped width, got {h}"
    );
}

#[test]
fn max_height_clamp_preserves_aspect_ratio_when_both_auto() {
    let style = Style {
        size: ui_layout::SizeStyle {
            max_height: LengthOrAuto::Length(Length::Percent(100.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    let (w, h) = resolve(&style, 1183.0, 683.0, Some(1183.0 / 683.0));
    assert!(
        (h - 300.0).abs() < 0.01,
        "height should clamp to 300, got {h}"
    );
    assert!(
        (w - 300.0 * (1183.0 / 683.0)).abs() < 0.01,
        "width should derive from the clamped height, got {w}"
    );
}

#[test]
fn percentage_width_resolves_against_containing_block() {
    let style = Style {
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Percent(50.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, None), (200.0, 100.0));
}

#[test]
fn vw_vh_units_resolve_against_viewport() {
    let style = Style {
        size: ui_layout::SizeStyle {
            width: LengthOrAuto::Length(Length::Vw(10.0)),
            height: LengthOrAuto::Length(Length::Vh(5.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(resolve(&style, 200.0, 100.0, None), (80.0, 30.0));
}
