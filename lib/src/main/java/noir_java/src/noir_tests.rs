use noir_rs::{
    native_types::{Witness, WitnessMap},
    FieldElement,
    AcirField,
    execute::execute,
    barretenberg::{
        prove::prove_ultra_honk,
        verify::verify_ultra_honk,
        srs::setup_srs,
    },
};
use std::collections::HashMap;
use crate::test_utils::{create_test_witness_map, TEST_CIRCUIT_BYTECODE, convert_to_witness_map};

#[test]
fn test_execute_circuit() {
    // Create a simple test case
    let mut input_map = HashMap::new();
    input_map.insert("0".to_string(), "0x3".to_string());  // x = 3
    input_map.insert("1".to_string(), "0x4".to_string());  // y = 4
    
    let witness_map = convert_to_witness_map(input_map);
    
    // Execute the circuit (x + y)
    let solved_witness = execute(TEST_CIRCUIT_BYTECODE, witness_map).expect("Circuit execution failed");
    
    // Get the return value (should be 7)
    let witness_map = &solved_witness.peek().into_iter().last().expect("No witness found").witness;
    
    // Print all witnesses for debugging
    println!("Witness map entries:");
    for w in witness_map.clone().into_iter() {
        println!("Witness {}: 0x{}", w.0 .0, w.1.to_hex());
    }
    
    // Check the last value in the witness map contains a value equal to 12 (3 * 4)
    assert_eq!(witness_map.clone().into_iter().last().expect("No result found").1.to_hex(), "000000000000000000000000000000000000000000000000000000000000000c", "The last value in the witness map should be 12");
}

#[test]
fn test_prove_verify_circuit() {

    // Create a simple test case
    let mut input_map = HashMap::new();
    input_map.insert("0".to_string(), "0x3".to_string());  // a = 3
    input_map.insert("1".to_string(), "0x4".to_string());  // b = 4
    
    let witness_map = convert_to_witness_map(input_map);
    
    // Set up SRS
    let _num_points = setup_srs(TEST_CIRCUIT_BYTECODE, None).expect("Failed to setup SRS");

    let vk = get_ultra_honk_verification_key(TEST_CIRCUIT_BYTECODE, false).expect("Failed to get verification key");
    
    // Generate a proof
    let proof = prove_ultra_honk(TEST_CIRCUIT_BYTECODE, witness_map, vk, false)
        .expect("Proof generation failed");
    
    // Verify the proof
    let verified = verify_ultra_honk(proof, vk).expect("Proof verification failed");
    assert!(verified, "Proof verification failed");
}

#[test]
fn test_witness_operations() {
    let mut witness_map = WitnessMap::new();
    
    // Insert some values
    witness_map.insert(Witness(0), FieldElement::try_from_str("0x3").unwrap());
    witness_map.insert(Witness(1), FieldElement::try_from_str("0x4").unwrap());
    
    // Check values
    assert_eq!(
        witness_map.get(&Witness(0)).unwrap().to_hex(),
        "0000000000000000000000000000000000000000000000000000000000000003"
    );
    assert_eq!(
        witness_map.get(&Witness(1)).unwrap().to_hex(),
        "0000000000000000000000000000000000000000000000000000000000000004"
    );
} 