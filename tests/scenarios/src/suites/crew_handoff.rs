use crate::registry::{Scenario, ScenarioOutcome, ValueArea};

pub fn scenarios() -> &'static [Scenario] {
    &SCENARIOS
}

static SCENARIOS: [Scenario; 3] = [
    Scenario {
        id: "S.4.1",
        name: "3D viewer on tablet",
        area: ValueArea::CrewHandoff,
        time_savings_minutes: 15.0,
        replaces: "Verbal walkthrough of what goes where, printed sketches",
        test_fn: s_4_1_tablet_viewer,
    },
    Scenario {
        id: "S.4.2",
        name: "DXF export",
        area: ValueArea::CrewHandoff,
        time_savings_minutes: 10.0,
        replaces: "Redrawing the design in CAD for crew reference",
        test_fn: s_4_2_dxf_export,
    },
    Scenario {
        id: "S.4.3",
        name: "Material callouts with supplier info",
        area: ValueArea::CrewHandoff,
        time_savings_minutes: 5.0,
        replaces: "Separate material list with SKUs and install specs",
        test_fn: s_4_3_material_callouts,
    },
];

fn s_4_1_tablet_viewer() -> ScenarioOutcome {
    // Validates: approved project → glTF loads in viewer → zones tappable →
    //           material info + dimensions displayed → works on iPad Safari
    // Requires: pt-scene, Bevy viewer, full stack
    ScenarioOutcome::NotImplemented
}

fn s_4_2_dxf_export() -> ScenarioOutcome {
    // Validates: approved project → DXF bytes → DXF contains correct layers
    //           per zone type, LWPOLYLINE entities match zone geometry,
    //           dimension annotations present, material labels in TEXT entities
    // Requires: pt-dxf, pt-project
    ScenarioOutcome::NotImplemented
}

fn s_4_3_material_callouts() -> ScenarioOutcome {
    // Validates: approved project → viewer shows material callouts per zone →
    //           callout includes product photo, supplier SKU, install depth
    // Requires: pt-materials, Bevy viewer or API endpoint
    ScenarioOutcome::NotImplemented
}
