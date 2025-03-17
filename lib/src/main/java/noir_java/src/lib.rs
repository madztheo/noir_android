use std::alloc::System;

use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jobject, jint, JNI_TRUE};
use jni::JNIEnv;
use noir_rs::{
    native_types::{Witness, WitnessMap},
    barretenberg::{
        prove::prove_ultra_honk,
        verify::verify_ultra_honk,
        srs::{setup_srs, setup_srs_from_bytecode},
        utils::get_honk_verification_key
    },
    FieldElement,
    AcirField,
    execute::execute,
};

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod noir_tests;

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_setup_1srs<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_size: jint,
    srs_path_jstr: JString<'local>,
) -> jint {
    let srs_path = match srs_path_jstr.is_null() {
        true => None,
        false => Some(
            env.get_string(&srs_path_jstr)
                .expect("Failed to get srs path string")
                .to_str()
                .expect("Failed to convert srs path to Rust string")
                .to_owned(),
        ),
    };

    let num_points = setup_srs(circuit_size.try_into().unwrap(), srs_path.as_deref()).expect("Failed to setup srs");

    jint::try_from(num_points).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_setup_1srs_1from_1bytecode<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    srs_path_jstr: JString<'local>,
    recursive: JString<'local>,
) -> jint {
    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .expect("Failed to get string from JString")
        .to_str()
        .expect("Failed to convert Java string to Rust string")
        .to_owned();

    let srs_path = match srs_path_jstr.is_null() {
        true => None,
        false => Some(
            env.get_string(&srs_path_jstr)
                .expect("Failed to get srs path string")
                .to_str()
                .expect("Failed to convert srs path to Rust string")
                .to_owned(),
        ),
    };

    let recursive_bool = env
        .get_string(&recursive)
        .expect("Failed to get string from JString")
        .to_str()
        .expect("Failed to convert recursive to Rust string")
        .to_owned() == "1";

    let num_points = setup_srs_from_bytecode(&circuit_bytecode, srs_path.as_deref(), recursive_bool).expect("Failed to setup srs");

    jint::try_from(num_points).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_execute<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    witness_jobject: JObject<'local>,
) -> jobject {
    // Use more descriptive variable names and handle errors gracefully
    let witness_map = match env.get_map(&witness_jobject) {
        Ok(map) => map,
        Err(e) => panic!("Failed to get witness map: {:?}", e),
    };
    let mut witness_iter = witness_map
        .iter(&mut env)
        .expect("Failed to create iterator");

    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .expect("Failed to get string from JString")
        .to_str()
        .expect("Failed to convert Java string to Rust string")
        .to_owned();

    let mut witness_map = WitnessMap::new();

    while let Ok(Some((key, value))) = witness_iter.next(&mut env) {
        let key_str = key.into();
        let value_str = value.into();

        let key_jstr = env.get_string(&key_str).expect("Failed to get key string");
        let value_jstr = env
            .get_string(&value_str)
            .expect("Failed to get value string");

        let key = key_jstr
            .to_str()
            .expect("Failed to convert key to Rust string");
        let value = value_jstr
            .to_str()
            .expect("Failed to convert value to Rust string");

        witness_map.insert(
            Witness(key.parse().expect("Failed to parse key")),
            FieldElement::try_from_str(value).expect("Failed to parse value"),
        );
    }

    let solved_witness = execute(&circuit_bytecode, witness_map).expect("Circuit execution failed");
    let witness_map = &solved_witness.peek().into_iter().last().expect("No witness found").witness;
    let witness_vec: Vec<String> = witness_map.clone().into_iter().map(|(_, val)| format!("0x{}", val.to_hex())).collect();

    // Create a Java String array - breaking down the operations to avoid multiple mutable borrows
    let string_class = env.find_class("java/lang/String").expect("Failed to find String class");
    let empty_string = env.new_string("").expect("Failed to create empty string");
    let string_array = env.new_object_array(
        witness_vec.len() as i32,
        string_class,
        &empty_string,
    ).expect("Failed to create string array");

    // Fill the array with witness values
    for (i, value) in witness_vec.iter().enumerate() {
        let jstring = env.new_string(value).expect("Failed to create Java string");
        env.set_object_array_element(&string_array, i as i32, &jstring)
            .expect("Failed to set array element");
    }

    string_array.as_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_prove<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    witness_jobject: JObject<'local>,
    proof_type_jstr: JString<'local>,
    recursive: JString<'local>,
) -> jobject {
    // Use more descriptive variable names and handle errors gracefully
    let witness_map = match env.get_map(&witness_jobject) {
        Ok(map) => map,
        Err(e) => panic!("Failed to get witness map: {:?}", e),
    };
    let mut witness_iter = witness_map
        .iter(&mut env)
        .expect("Failed to create iterator");

    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .expect("Failed to get string from JString")
        .to_str()
        .expect("Failed to convert Java string to Rust string")
        .to_owned();

    let proof_type = env
        .get_string(&proof_type_jstr)
        .expect("Failed to get proof type string")
        .to_str()
        .expect("Failed to convert proof type to Rust string")
        .to_owned();

    let recursive_bool = env
        .get_string(&recursive)
        .expect("Failed to get string from JString")
        .to_str()
        .expect("Failed to convert recursive to Rust string")
        .to_owned() == "1";

    let mut witness_map = WitnessMap::new();

    while let Ok(Some((key, value))) = witness_iter.next(&mut env) {
        let key_str = key.into();
        let value_str = value.into();

        let key_jstr = env.get_string(&key_str).expect("Failed to get key string");
        let value_jstr = env
            .get_string(&value_str)
            .expect("Failed to get value string");

        let key = key_jstr
            .to_str()
            .expect("Failed to convert key to Rust string");
        let value = value_jstr
            .to_str()
            .expect("Failed to convert value to Rust string");

        witness_map.insert(
            Witness(key.parse().expect("Failed to parse key")),
            FieldElement::try_from_str(value).expect("Failed to parse value"),
        );
    }

    let proof = if proof_type == "honk" { 
        prove_ultra_honk(&circuit_bytecode, witness_map, recursive_bool).expect("Proof generation failed") 
    } else { 
        panic!("Honk is the only proof type supported for now");
    };

    let proof_str = hex::encode(proof);

    // Create and return a Java string containing the proof
    let proof_jstr = env
        .new_string(proof_str)
        .expect("Failed to create Java string for proof");

    proof_jstr.into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_verify<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    proof_jstr: JString<'local>,
    vk_jstr: JString<'local>,
    proof_type_jstr: JString<'local>
) -> jboolean {
    let proof_str = env
        .get_string(&proof_jstr)
        .expect("Failed to get proof string")
        .to_str()
        .expect("Failed to convert proof to Rust string")
        .to_owned();

    let vk_str = env
        .get_string(&vk_jstr)
        .expect("Failed to get verification key string")
        .to_str()
        .expect("Failed to convert verification key to Rust string")
        .to_owned();

    let proof = hex::decode(proof_str).expect("Failed to decode proof");
    let verification_key = hex::decode(vk_str).expect("Failed to decode verification key");

    let proof_type = env
        .get_string(&proof_type_jstr)
        .expect("Failed to get proof type string")
        .to_str()
        .expect("Failed to convert proof type to Rust string")
        .to_owned();

    let verdict = if proof_type == "honk" {
        verify_ultra_honk(proof, verification_key).expect("Verification failed")
    } else {
        panic!("Ultra honk is the only proof type supported for now");
    };

    jboolean::from(verdict)
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_get_1verification_1key<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    recursive: JString<'local>,
) -> jobject {
    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .expect("Failed to get string from JString")
        .to_str()
        .expect("Failed to convert Java string to Rust string")
        .to_owned();

    let recursive_bool = env
        .get_string(&recursive)
        .expect("Failed to get string from JString")
        .to_str()
        .expect("Failed to convert recursive to Rust string")
        .to_owned() == "1";

    let vk = get_honk_verification_key(&circuit_bytecode, recursive_bool).expect("Failed to get verification key");

    let vk_str = hex::encode(vk);

    // Create and return a Java string containing the proof
    let vk_jstr = env
        .new_string(vk_str)
        .expect("Failed to create Java string for vk");

    vk_jstr.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock JNI environment for testing
    struct MockJNIEnv;

    impl MockJNIEnv {
        fn new() -> Self {
            MockJNIEnv {}
        }
    }

    // Test helper functions separate from the JNI interface
    #[test]
    fn test_field_element_conversion() {
        let hex_value = "0x1234";
        let field_element = FieldElement::try_from_str(hex_value).expect("Failed to convert hex to field element");
        // The to_hex method returns the full 64-character hex representation with leading zeros
        assert_eq!(format!("0x{:0>64}", field_element.to_hex()), format!("0x{:0>64}", "1234"));
    }

    #[test]
    fn test_witness_map_operations() {
        let mut witness_map = WitnessMap::new();
        witness_map.insert(
            Witness(0),
            FieldElement::try_from_str("0x1234").expect("Failed to convert"),
        );
        witness_map.insert(
            Witness(1),
            FieldElement::try_from_str("0x5678").expect("Failed to convert"),
        );

        assert!(witness_map.contains_key(&Witness(0)));
        assert!(witness_map.contains_key(&Witness(1)));
        assert!(!witness_map.contains_key(&Witness(2)));
    }
}
