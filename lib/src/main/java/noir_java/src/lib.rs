use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jobject, jint};
use jni::JNIEnv;
use noir_rs::{
    native_types::{Witness, WitnessMap},
    barretenberg::{
        prove::{prove_ultra_honk, prove_ultra_honk_keccak},
        verify::{verify_ultra_honk, get_ultra_honk_verification_key, verify_ultra_honk_keccak, get_ultra_honk_keccak_verification_key},
        srs::{setup_srs, setup_srs_from_bytecode},
    },
    FieldElement,
    AcirField,
    execute::execute,
};
use log::{info, error, debug};

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod noir_tests;

// Initialize Android logger if not already initialized
fn init_logger() {
    #[cfg(target_os = "android")]
    {
        use android_logger::{Config, FilterBuilder};
        use log::LevelFilter;

        static LOGGER_INITIALIZED: std::sync::Once = std::sync::Once::new();
        
        LOGGER_INITIALIZED.call_once(|| {
            android_logger::init_once(
                Config::default()
                    .with_max_level(LevelFilter::Debug)
                    .with_tag("NoirAndroid")
                    .with_filter(FilterBuilder::new().parse("debug").build()),
            );
            debug!("Android logger initialized for NoirAndroid");
        });
    }

    #[cfg(not(target_os = "android"))]
    {
        use env_logger;
        static LOGGER_INITIALIZED: std::sync::Once = std::sync::Once::new();
        
        LOGGER_INITIALIZED.call_once(|| {
            env_logger::init();
            debug!("Env logger initialized for NoirAndroid");
        });
    }
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_setup_1srs<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_size: jint,
    srs_path_jstr: JString<'local>,
) -> jint {
    init_logger();
    info!("Setting up SRS with circuit size: {}", circuit_size);
    
    let srs_path = match srs_path_jstr.is_null() {
        true => {
            debug!("SRS path is null, using default");
            None
        },
        false => {
            let path = env.get_string(&srs_path_jstr)
                .map_err(|e| {
                    error!("Failed to get srs path string: {:?}", e);
                    e
                })
                .expect("Failed to get srs path string")
                .to_str()
                .map_err(|e| {
                    error!("Failed to convert srs path to Rust string: {:?}", e);
                    e
                })
                .expect("Failed to convert srs path to Rust string")
                .to_owned();
            debug!("Using SRS path: {}", path);
            Some(path)
        },
    };

    let num_points = match setup_srs(circuit_size.try_into().unwrap(), srs_path.as_deref()) {
        Ok(num) => {
            info!("SRS setup successful with {} points", num);
            num
        },
        Err(e) => {
            error!("Failed to setup SRS: {:?}", e);
            panic!("Failed to setup SRS: {:?}", e);
        }
    };

    jint::try_from(num_points).unwrap_or_else(|e| {
        error!("Failed to convert num_points to jint: {:?}", e);
        panic!("Failed to convert num_points to jint: {:?}", e)
    })
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_setup_1srs_1from_1bytecode<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    srs_path_jstr: JString<'local>
) -> jint {
    init_logger();
    debug!("Setting up SRS from bytecode");
    
    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .map_err(|e| {
            error!("Failed to get bytecode string: {:?}", e);
            e
        })
        .expect("Failed to get string from JString")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert Java string to Rust string")
        .to_owned();
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());

    let srs_path = match srs_path_jstr.is_null() {
        true => {
            debug!("SRS path is null, using default");
            None
        },
        false => {
            let path = env.get_string(&srs_path_jstr)
                .map_err(|e| {
                    error!("Failed to get srs path string: {:?}", e);
                    e
                })
                .expect("Failed to get srs path string")
                .to_str()
                .map_err(|e| {
                    error!("Failed to convert srs path to Rust string: {:?}", e);
                    e
                })
                .expect("Failed to convert srs path to Rust string")
                .to_owned();
            debug!("Using SRS path: {}", path);
            Some(path)
        },
    };

    let num_points = match setup_srs_from_bytecode(&circuit_bytecode, srs_path.as_deref(), false) {
        Ok(num) => {
            info!("SRS setup from bytecode successful with {} points", num);
            num
        },
        Err(e) => {
            error!("Failed to setup SRS from bytecode: {:?}", e);
            panic!("Failed to setup SRS from bytecode: {:?}", e);
        }
    };

    jint::try_from(num_points).unwrap_or_else(|e| {
        error!("Failed to convert num_points to jint: {:?}", e);
        panic!("Failed to convert num_points to jint: {:?}", e)
    })
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_execute<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    witness_jobject: JObject<'local>,
) -> jobject {
    init_logger();
    info!("Executing circuit");
    
    let witness_map = match env.get_map(&witness_jobject) {
        Ok(map) => {
            debug!("Successfully retrieved witness map from Java");
            map
        },
        Err(e) => {
            error!("Failed to get witness map: {:?}", e);
            panic!("Failed to get witness map: {:?}", e)
        },
    };
    let mut witness_iter = match witness_map.iter(&mut env) {
        Ok(iter) => iter,
        Err(e) => {
            error!("Failed to create iterator for witness map: {:?}", e);
            panic!("Failed to create iterator: {:?}", e)
        }
    };

    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .map_err(|e| {
            error!("Failed to get bytecode string: {:?}", e);
            e
        })
        .expect("Failed to get string from JString")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert Java string to Rust string")
        .to_owned();
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());

    let mut witness_map = WitnessMap::new();
    let mut witness_count = 0;

    while let Ok(Some((key, value))) = witness_iter.next(&mut env) {
        witness_count += 1;
        let key_str = key.into();
        let value_str = value.into();

        let key_jstr = env.get_string(&key_str)
            .map_err(|e| {
                error!("Failed to get key string: {:?}", e);
                e
            })
            .expect("Failed to get key string");
        let value_jstr = env
            .get_string(&value_str)
            .map_err(|e| {
                error!("Failed to get value string: {:?}", e);
                e
            })
            .expect("Failed to get value string");

        let key = key_jstr
            .to_str()
            .map_err(|e| {
                error!("Failed to convert key to Rust string: {:?}", e);
                e
            })
            .expect("Failed to convert key to Rust string");
        let value = value_jstr
            .to_str()
            .map_err(|e| {
                error!("Failed to convert value to Rust string: {:?}", e);
                e
            })
            .expect("Failed to convert value to Rust string");
        
        let witness_key = match key.parse() {
            Ok(k) => Witness(k),
            Err(e) => {
                error!("Failed to parse witness key '{}': {:?}", key, e);
                panic!("Failed to parse key: {:?}", e)
            }
        };
        
        let field_element = match FieldElement::try_from_str(value).unwrap_or_else(|| {
            error!("Failed to parse witness value '{}': not a valid field element", value);
            panic!("Failed to parse value: not a valid field element")
        }) {
            fe => fe,
        };

        witness_map.insert(witness_key, field_element);
    }
    
    info!("Loaded {} witness values", witness_count);

    let solved_witness = match execute(&circuit_bytecode, witness_map) {
        Ok(witness) => {
            info!("Circuit execution successful");
            witness
        },
        Err(e) => {
            error!("Circuit execution failed: {:?}", e);
            panic!("Circuit execution failed: {:?}", e)
        }
    };
    
    let witness_map = match solved_witness.peek().into_iter().last() {
        Some(w) => {
            debug!("Successfully retrieved witness from execution result");
            &w.witness
        },
        None => {
            error!("No witness found in execution result");
            panic!("No witness found")
        }
    };
    
    let witness_vec: Vec<String> = witness_map.clone().into_iter().map(|(_, val)| format!("0x{}", val.to_hex())).collect();
    debug!("Generated {} witness values", witness_vec.len());

    // Create a Java String array - breaking down the operations to avoid multiple mutable borrows
    let string_class = match env.find_class("java/lang/String") {
        Ok(class) => class,
        Err(e) => {
            error!("Failed to find String class: {:?}", e);
            panic!("Failed to find String class: {:?}", e)
        }
    };
    
    let empty_string = match env.new_string("") {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create empty string: {:?}", e);
            panic!("Failed to create empty string: {:?}", e)
        }
    };
    
    let string_array = match env.new_object_array(
        witness_vec.len() as i32,
        string_class,
        &empty_string,
    ) {
        Ok(arr) => arr,
        Err(e) => {
            error!("Failed to create string array: {:?}", e);
            panic!("Failed to create string array: {:?}", e)
        }
    };

    // Fill the array with witness values
    for (i, value) in witness_vec.iter().enumerate() {
        let jstring = match env.new_string(value) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create Java string for witness value {}: {:?}", i, e);
                panic!("Failed to create Java string: {:?}", e)
            }
        };
        
        if let Err(e) = env.set_object_array_element(&string_array, i as i32, &jstring) {
            error!("Failed to set array element at index {}: {:?}", i, e);
            panic!("Failed to set array element: {:?}", e);
        }
    }

    info!("Successfully prepared witness array with {} elements", witness_vec.len());
    string_array.as_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_prove<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    witness_jobject: JObject<'local>,
    vk_jstr: JString<'local>,
    proof_type_jstr: JString<'local>,
) -> jobject {
    init_logger();
    info!("Starting proof generation");
    
    // Use more descriptive variable names and handle errors gracefully
    let witness_map = match env.get_map(&witness_jobject) {
        Ok(map) => {
            debug!("Successfully retrieved witness map from Java");
            map
        },
        Err(e) => {
            error!("Failed to get witness map: {:?}", e);
            panic!("Failed to get witness map: {:?}", e)
        },
    };
    let mut witness_iter = match witness_map.iter(&mut env) {
        Ok(iter) => iter,
        Err(e) => {
            error!("Failed to create iterator for witness map: {:?}", e);
            panic!("Failed to create iterator: {:?}", e)
        }
    };

    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .map_err(|e| {
            error!("Failed to get bytecode string: {:?}", e);
            e
        })
        .expect("Failed to get string from JString")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert Java string to Rust string")
        .to_owned();
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());

    let proof_type = env
        .get_string(&proof_type_jstr)
        .map_err(|e| {
            error!("Failed to get proof type string: {:?}", e);
            e
        })
        .expect("Failed to get proof type string")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert proof type to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert proof type to Rust string")
        .to_owned();
    info!("Using proof type: {}", proof_type);

    let vk_str = env
        .get_string(&vk_jstr)
        .map_err(|e| {
            error!("Failed to get verification key string: {:?}", e);
            e
        })
        .expect("Failed to get verification key string")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert verification key to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert verification key to Rust string")
        .to_owned();
    debug!("Verification key length: {}", vk_str.len());

    let verification_key = match hex::decode(vk_str) {
        Ok(vk) => {
            debug!("Successfully decoded verification key, size: {} bytes", vk.len());
            vk
        },
        Err(e) => {
            error!("Failed to decode verification key: {:?}", e);
            panic!("Failed to decode verification key: {:?}", e)
        }
    };

    let mut witness_map = WitnessMap::new();
    let mut witness_count = 0;

    while let Ok(Some((key, value))) = witness_iter.next(&mut env) {
        witness_count += 1;
        let key_str = key.into();
        let value_str = value.into();

        let key_jstr = env.get_string(&key_str)
            .map_err(|e| {
                error!("Failed to get key string: {:?}", e);
                e
            })
            .expect("Failed to get key string");
        let value_jstr = env
            .get_string(&value_str)
            .map_err(|e| {
                error!("Failed to get value string: {:?}", e);
                e
            })
            .expect("Failed to get value string");

        let key = key_jstr
            .to_str()
            .map_err(|e| {
                error!("Failed to convert key to Rust string: {:?}", e);
                e
            })
            .expect("Failed to convert key to Rust string");
        let value = value_jstr
            .to_str()
            .map_err(|e| {
                error!("Failed to convert value to Rust string: {:?}", e);
                e
            })
            .expect("Failed to convert value to Rust string");
        
        let witness_key = match key.parse() {
            Ok(k) => Witness(k),
            Err(e) => {
                error!("Failed to parse witness key '{}': {:?}", key, e);
                panic!("Failed to parse key: {:?}", e)
            }
        };
        
        let field_element = match FieldElement::try_from_str(value).unwrap_or_else(|| {
            error!("Failed to parse witness value '{}': not a valid field element", value);
            panic!("Failed to parse value: not a valid field element")
        }) {
            fe => fe,
        };

        witness_map.insert(witness_key, field_element);
    }
    
    info!("Loaded {} witness values", witness_count);

    let proof = if proof_type == "ultra_honk" { 
        info!("Generating UltraHonk proof");
        match prove_ultra_honk(&circuit_bytecode, witness_map, verification_key) {
            Ok(p) => {
                info!("Proof generation successful, proof size: {} bytes", p.len());
                p
            },
            Err(e) => {
                error!("Proof generation failed: {:?}", e);
                panic!("Proof generation failed: {:?}", e)
            }
        }
    } else if proof_type == "ultra_honk_keccak" {
        info!("Generating UltraHonkKeccak proof");
        match prove_ultra_honk_keccak(&circuit_bytecode, witness_map, verification_key, false) {
            Ok(p) => {
                info!("Proof generation successful, proof size: {} bytes", p.len());
                p
            },
            Err(e) => {
                error!("Proof generation failed: {:?}", e);
                panic!("Proof generation failed: {:?}", e)
            }
        }
    } else { 
        error!("Unsupported proof type: {}", proof_type);
        panic!("Unsupported proof type: {}", proof_type)
    };

    let proof_str = hex::encode(&proof);
    debug!("Encoded proof length: {}", proof_str.len());

    // Create and return a Java string containing the proof
    let proof_jstr = match env.new_string(proof_str) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create Java string for proof: {:?}", e);
            panic!("Failed to create Java string for proof: {:?}", e)
        }
    };

    info!("Successfully prepared proof for return to Java");
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
    init_logger();
    info!("Starting proof verification");
    
    let proof_str = env
        .get_string(&proof_jstr)
        .map_err(|e| {
            error!("Failed to get proof string: {:?}", e);
            e
        })
        .expect("Failed to get proof string")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert proof to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert proof to Rust string")
        .to_owned();
    debug!("Proof string length: {}", proof_str.len());

    let vk_str = env
        .get_string(&vk_jstr)
        .map_err(|e| {
            error!("Failed to get verification key string: {:?}", e);
            e
        })
        .expect("Failed to get verification key string")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert verification key to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert verification key to Rust string")
        .to_owned();
    debug!("Verification key length: {}", vk_str.len());

    let proof = match hex::decode(proof_str) {
        Ok(p) => {
            debug!("Successfully decoded proof, size: {} bytes", p.len());
            p
        },
        Err(e) => {
            error!("Failed to decode proof: {:?}", e);
            panic!("Failed to decode proof: {:?}", e)
        }
    };
    
    let verification_key = match hex::decode(vk_str) {
        Ok(vk) => {
            debug!("Successfully decoded verification key, size: {} bytes", vk.len());
            vk
        },
        Err(e) => {
            error!("Failed to decode verification key: {:?}", e);
            panic!("Failed to decode verification key: {:?}", e)
        }
    };

    let proof_type = env
        .get_string(&proof_type_jstr)
        .map_err(|e| {
            error!("Failed to get proof type string: {:?}", e);
            e
        })
        .expect("Failed to get proof type string")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert proof type to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert proof type to Rust string")
        .to_owned();
    info!("Using proof type: {}", proof_type);

    let verdict = if proof_type == "ultra_honk" {
        match verify_ultra_honk(proof, verification_key) {
            Ok(v) => {
                info!("Verification complete, result: {}", v);
                v
            },  
            Err(e) => {
                error!("Verification failed with error: {:?}", e);
                panic!("Verification failed: {:?}", e)
            }
        }
    } else if proof_type == "ultra_honk_keccak" {
        match verify_ultra_honk_keccak(proof, verification_key, false) {
            Ok(v) => {
                info!("Verification complete, result: {}", v);
                v
            },
            Err(e) => {
                error!("Verification failed with error: {:?}", e);
                panic!("Verification failed: {:?}", e)
            }
        }
    } else {
        error!("Unsupported proof type: {}", proof_type);
        panic!("Ultra honk and Ultra honk keccak are the only proof types supported for now");
    };

    jboolean::from(verdict)
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_get_1verification_1key<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    proof_type_jstr: JString<'local>
) -> jobject {
    init_logger();
    info!("Getting verification key");
    
    let circuit_bytecode = env
        .get_string(&circuit_bytecode_jstr)
        .map_err(|e| {
            error!("Failed to get bytecode string: {:?}", e);
            e
        })
        .expect("Failed to get string from JString")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert Java string to Rust string")
        .to_owned();
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());


    let proof_type = env
        .get_string(&proof_type_jstr)
        .map_err(|e| {
            error!("Failed to get proof type string: {:?}", e);
            e
        })
        .expect("Failed to get proof type string")
        .to_str()
        .map_err(|e| {
            error!("Failed to convert proof type to Rust string: {:?}", e);
            e
        })
        .expect("Failed to convert proof type to Rust string")
        .to_owned();
    info!("Using proof type: {}", proof_type);

    let vk = if proof_type == "ultra_honk" {
        match get_ultra_honk_verification_key(&circuit_bytecode) {
            Ok(key) => {
                info!("Successfully retrieved verification key, size: {} bytes", key.len());
                key
            },
            Err(e) => {
                error!("Failed to get verification key: {:?}", e);
                panic!("Failed to get verification key: {:?}", e)
            }
        }
    } else if proof_type == "ultra_honk_keccak" {
        match get_ultra_honk_keccak_verification_key(&circuit_bytecode, false) {
            Ok(key) => {
                info!("Successfully retrieved verification key, size: {} bytes", key.len());
                key
            },
            Err(e) => {
                error!("Failed to get verification key: {:?}", e);
                panic!("Failed to get verification key: {:?}", e)
            }
        }
    } else {
        error!("Unsupported proof type: {}", proof_type);
        panic!("Ultra honk and Ultra honk keccak are the only proof types supported for now");
    };

    let vk_str = hex::encode(&vk);
    debug!("Encoded verification key length: {}", vk_str.len());

    // Create and return a Java string containing the proof
    let vk_jstr = match env.new_string(vk_str) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create Java string for verification key: {:?}", e);
            panic!("Failed to create Java string for vk: {:?}", e)
        }
    };

    info!("Successfully prepared verification key for return to Java");
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
