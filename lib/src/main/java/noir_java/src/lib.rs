use std::alloc::System;

use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jobject, jint, JNI_TRUE};
use jni::JNIEnv;
use noir_rs::{
    native_types::{Witness, WitnessMap},
    barretenberg::{
        prove::prove_ultra_honk,
        verify::verify_ultra_honk,
        srs::setup_srs,
    },
    FieldElement
};

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_setup_1srs<'local>(
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

    let num_points = setup_srs(&circuit_bytecode, srs_path.as_deref(), recursive_bool).expect("Failed to setup srs");

    jint::try_from(num_points).unwrap()
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

    let (proof, vk) = if proof_type == "honk" { 
        prove_ultra_honk(&circuit_bytecode, witness_map, recursive_bool).expect("Proof generation failed") 
    } else { 
        panic!("Honk is the only proof type supported for now");
    };

    let proof_str = hex::encode(proof);
    let vk_str = hex::encode(vk);

    let proof_jstr = env
        .new_string(proof_str)
        .expect("Failed to create Java string for proof");
    let vk_jstr = env
        .new_string(vk_str)
        .expect("Failed to create Java string for vk");

    let proof_class = env
        .find_class("com/noirandroid/lib/Proof")
        .expect("Failed to find Proof class");
    env.new_object(
        proof_class,
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[(&proof_jstr).into(), (&vk_jstr).into()],
    )
    .expect("Failed to create new Proof object")
    .as_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_verify<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    mut proof_jobject: JObject<'local>,
    proof_type_jstr: JString<'local>
) -> jboolean {
    let proof_field = env
        .get_field(&mut proof_jobject, "proof", "Ljava/lang/String;")
        .expect("Failed to get proof field")
        .l()
        .expect("Failed to get proof object");
    let proof_str: JString = proof_field.into();
    let proof_jstr = env
        .get_string(&proof_str)
        .expect("Failed to get string from JString");
    let proof_str = proof_jstr
        .to_str()
        .expect("Failed to convert Java string to Rust string");

    let vk_field = env
        .get_field(&mut proof_jobject, "vk", "Ljava/lang/String;")
        .expect("Failed to get vk field")
        .l()
        .expect("Failed to get vk object");

    let vk_str: JString = vk_field.into();
    let vk_jstr = env
        .get_string(&vk_str)
        .expect("Failed to get string from JString");
    let vk_str = vk_jstr
        .to_str()
        .expect("Failed to convert Java string to Rust string");

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
