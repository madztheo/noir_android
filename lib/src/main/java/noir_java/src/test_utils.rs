use noir_rs::{
    native_types::{Witness, WitnessMap},
    FieldElement,
    AcirField,
};
use std::collections::HashMap;

/// Creates a simple witness map for testing
pub fn create_test_witness_map() -> WitnessMap<FieldElement> {
    let mut witness_map = WitnessMap::new();
    witness_map.insert(
        Witness(0),
        FieldElement::try_from_str("0x1234").expect("Failed to convert"),
    );
    witness_map.insert(
        Witness(1),
        FieldElement::try_from_str("0x5678").expect("Failed to convert"),
    );
    witness_map
}

/// Simple addition circuit for testing
pub const TEST_CIRCUIT_BYTECODE: &str = "H4sIAAAAAAAA/42NsQmAMBBF74KDWGqnOIIIVmJpYyHYWChiZ5kRxAWcQnScdJY29gZNSAgp8or7x93/fIQfT2jfdAPhiqCQuw9OoBxmLmqLicVbeJTZTmlVB8mVz+e4pOxZb/4n7h2fVy9Ey93kBZmTjiLsAAAA";

/// Converts a HashMap<String, String> to a WitnessMap
pub fn convert_to_witness_map(map: HashMap<String, String>) -> WitnessMap<FieldElement> {
    let mut witness_map = WitnessMap::new();
    for (key, value) in map {
        witness_map.insert(
            Witness(key.parse().expect("Failed to parse witness index")),
            FieldElement::try_from_str(&value).expect("Failed to convert value"),
        );
    }
    witness_map
} 