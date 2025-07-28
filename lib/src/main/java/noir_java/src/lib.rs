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

// Helper function to throw Java exceptions
fn throw_exception(env: &mut JNIEnv, exception_class: &str, message: &str) {
    if let Err(e) = env.throw_new(exception_class, message) {
        error!("Failed to throw Java exception: {:?}", e);
    }
}

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
            let path = match env.get_string(&srs_path_jstr) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to get srs path string: {:?}", e);
                    throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get srs path string: {:?}", e));
                    return -1;
                }
            };
            let path_str = match path.to_str() {
                Ok(s) => s.to_owned(),
                Err(e) => {
                    error!("Failed to convert srs path to Rust string: {:?}", e);
                    throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert srs path to Rust string: {:?}", e));
                    return -1;
                }
            };
            debug!("Using SRS path: {}", path_str);
            Some(path_str)
        },
    };

    let num_points = match setup_srs(circuit_size.try_into().unwrap(), srs_path.as_deref()) {
        Ok(num) => {
            info!("SRS setup successful with {} points", num);
            num
        },
        Err(e) => {
            error!("Failed to setup SRS: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to setup SRS: {:?}", e));
            return -1;
        }
    };

    match jint::try_from(num_points) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to convert num_points to jint: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert num_points to jint: {:?}", e));
            -1
        }
    }
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
    
    let circuit_bytecode = match env.get_string(&circuit_bytecode_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get bytecode string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get bytecode string: {:?}", e));
            return -1;
        }
    };
    let circuit_bytecode = match circuit_bytecode.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert bytecode to Rust string: {:?}", e));
            return -1;
        }
    };
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());

    let srs_path = match srs_path_jstr.is_null() {
        true => {
            debug!("SRS path is null, using default");
            None
        },
        false => {
            let path = match env.get_string(&srs_path_jstr) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to get srs path string: {:?}", e);
                    throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get srs path string: {:?}", e));
                    return -1;
                }
            };
            let path_str = match path.to_str() {
                Ok(s) => s.to_owned(),
                Err(e) => {
                    error!("Failed to convert srs path to Rust string: {:?}", e);
                    throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert srs path to Rust string: {:?}", e));
                    return -1;
                }
            };
            debug!("Using SRS path: {}", path_str);
            Some(path_str)
        },
    };

    let num_points = match setup_srs_from_bytecode(&circuit_bytecode, srs_path.as_deref(), false) {
        Ok(num) => {
            info!("SRS setup from bytecode successful with {} points", num);
            num
        },
        Err(e) => {
            error!("Failed to setup SRS from bytecode: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to setup SRS from bytecode: {:?}", e));
            return -1;
        }
    };

    match jint::try_from(num_points) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to convert num_points to jint: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert num_points to jint: {:?}", e));
            -1
        }
    }
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
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get witness map: {:?}", e));
            return std::ptr::null_mut();
        },
    };
    let mut witness_iter = match witness_map.iter(&mut env) {
        Ok(iter) => iter,
        Err(e) => {
            error!("Failed to create iterator for witness map: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to create iterator: {:?}", e));
            return std::ptr::null_mut();
        }
    };

    let circuit_bytecode = match env.get_string(&circuit_bytecode_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get bytecode string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get bytecode string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    let circuit_bytecode = match circuit_bytecode.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert bytecode to Rust string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());

    let mut witness_map = WitnessMap::new();
    let mut witness_count = 0;

    while let Ok(Some((key, value))) = witness_iter.next(&mut env) {
        witness_count += 1;
        let key_str = key.into();
        let value_str = value.into();

        let key_jstr = match env.get_string(&key_str) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get key string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get key string: {:?}", e));
                return std::ptr::null_mut();
            }
        };
        let value_jstr = match env.get_string(&value_str) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get value string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get value string: {:?}", e));
                return std::ptr::null_mut();
            }
        };

        let key = match key_jstr.to_str() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to convert key to Rust string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert key to Rust string: {:?}", e));
                return std::ptr::null_mut();
            }
        };
        let value = match value_jstr.to_str() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to convert value to Rust string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert value to Rust string: {:?}", e));
                return std::ptr::null_mut();
            }
        };
        
        let witness_key = match key.parse() {
            Ok(k) => Witness(k),
            Err(e) => {
                error!("Failed to parse witness key '{}': {:?}", key, e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to parse witness key '{}': {:?}", key, e));
                return std::ptr::null_mut();
            }
        };
        
        let field_element = match FieldElement::try_from_str(value) {
            Some(fe) => fe,
            None => {
                error!("Failed to parse witness value '{}': not a valid field element", value);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to parse witness value '{}': not a valid field element", value));
                return std::ptr::null_mut();
            }
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
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Circuit execution failed: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    
    let witness_map = match solved_witness.peek().into_iter().last() {
        Some(w) => {
            debug!("Successfully retrieved witness from execution result");
            &w.witness
        },
        None => {
            error!("No witness found in execution result");
            throw_exception(&mut env, "java/lang/RuntimeException", "No witness found in execution result");
            return std::ptr::null_mut();
        }
    };
    
    let witness_vec: Vec<String> = witness_map.clone().into_iter().map(|(_, val)| format!("0x{}", val.to_hex())).collect();
    debug!("Generated {} witness values", witness_vec.len());

    // Create a Java String array - breaking down the operations to avoid multiple mutable borrows
    let string_class = match env.find_class("java/lang/String") {
        Ok(class) => class,
        Err(e) => {
            error!("Failed to find String class: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to find String class: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    
    let empty_string = match env.new_string("") {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create empty string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to create empty string: {:?}", e));
            return std::ptr::null_mut();
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
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to create string array: {:?}", e));
            return std::ptr::null_mut();
        }
    };

    // Fill the array with witness values
    for (i, value) in witness_vec.iter().enumerate() {
        let jstring = match env.new_string(value) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create Java string for witness value {}: {:?}", i, e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to create Java string for witness value {}: {:?}", i, e));
                return std::ptr::null_mut();
            }
        };
        
        if let Err(e) = env.set_object_array_element(&string_array, i as i32, &jstring) {
            error!("Failed to set array element at index {}: {:?}", i, e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to set array element at index {}: {:?}", i, e));
            return std::ptr::null_mut();
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
    low_memory_mode: jboolean
) -> jobject {
    init_logger();
    info!("Starting proof generation");
    
    let use_low_memory = low_memory_mode != 0;
    debug!("Low memory mode: {}", use_low_memory);
    
    // Use more descriptive variable names and handle errors gracefully
    let witness_map = match env.get_map(&witness_jobject) {
        Ok(map) => {
            debug!("Successfully retrieved witness map from Java");
            map
        },
        Err(e) => {
            error!("Failed to get witness map: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get witness map: {:?}", e));
            return std::ptr::null_mut();
        },
    };
    let mut witness_iter = match witness_map.iter(&mut env) {
        Ok(iter) => iter,
        Err(e) => {
            error!("Failed to create iterator for witness map: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to create iterator: {:?}", e));
            return std::ptr::null_mut();
        }
    };

    let circuit_bytecode = match env.get_string(&circuit_bytecode_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get bytecode string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get bytecode string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    let circuit_bytecode = match circuit_bytecode.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert bytecode to Rust string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());

    let proof_type = match env.get_string(&proof_type_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get proof type string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get proof type string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    let proof_type = match proof_type.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert proof type to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert proof type to Rust string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    info!("Using proof type: {}", proof_type);

    let vk_str = match env.get_string(&vk_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get verification key string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get verification key string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    let vk_str = match vk_str.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert verification key to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert verification key to Rust string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    debug!("Verification key length: {}", vk_str.len());

    let verification_key = match hex::decode(vk_str) {
        Ok(vk) => {
            debug!("Successfully decoded verification key, size: {} bytes", vk.len());
            vk
        },
        Err(e) => {
            error!("Failed to decode verification key: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to decode verification key: {:?}", e));
            return std::ptr::null_mut();
        }
    };

    let mut witness_map = WitnessMap::new();
    let mut witness_count = 0;

    while let Ok(Some((key, value))) = witness_iter.next(&mut env) {
        witness_count += 1;
        let key_str = key.into();
        let value_str = value.into();

        let key_jstr = match env.get_string(&key_str) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get key string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get key string: {:?}", e));
                return std::ptr::null_mut();
            }
        };
        let value_jstr = match env.get_string(&value_str) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get value string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get value string: {:?}", e));
                return std::ptr::null_mut();
            }
        };

        let key = match key_jstr.to_str() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to convert key to Rust string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert key to Rust string: {:?}", e));
                return std::ptr::null_mut();
            }
        };
        let value = match value_jstr.to_str() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to convert value to Rust string: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert value to Rust string: {:?}", e));
                return std::ptr::null_mut();
            }
        };
        
        let witness_key = match key.parse() {
            Ok(k) => Witness(k),
            Err(e) => {
                error!("Failed to parse witness key '{}': {:?}", key, e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to parse witness key '{}': {:?}", key, e));
                return std::ptr::null_mut();
            }
        };
        
        let field_element = match FieldElement::try_from_str(value) {
            Some(fe) => fe,
            None => {
                error!("Failed to parse witness value '{}': not a valid field element", value);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to parse witness value '{}': not a valid field element", value));
                return std::ptr::null_mut();
            }
        };

        witness_map.insert(witness_key, field_element);
    }
    
    info!("Loaded {} witness values", witness_count);

    let proof = if proof_type == "ultra_honk" { 
        info!("Generating UltraHonk proof");
        match prove_ultra_honk(&circuit_bytecode, witness_map, verification_key, use_low_memory) {
            Ok(p) => {
                info!("Proof generation successful, proof size: {} bytes", p.len());
                p
            },
            Err(e) => {
                error!("Proof generation failed: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Proof generation failed: {:?}", e));
                return std::ptr::null_mut();
            }
        }
    } else if proof_type == "ultra_honk_keccak" {
        info!("Generating UltraHonkKeccak proof");
        match prove_ultra_honk_keccak(&circuit_bytecode, witness_map, verification_key, false, use_low_memory) {
            Ok(p) => {
                info!("Proof generation successful, proof size: {} bytes", p.len());
                p
            },
            Err(e) => {
                error!("Proof generation failed: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Proof generation failed: {:?}", e));
                return std::ptr::null_mut();
            }
        }
    } else { 
        error!("Unsupported proof type: {}", proof_type);
        throw_exception(&mut env, "java/lang/IllegalArgumentException", &format!("Unsupported proof type: {}", proof_type));
        return std::ptr::null_mut();
    };

    let proof_str = hex::encode(&proof);
    debug!("Encoded proof length: {}", proof_str.len());

    // Create and return a Java string containing the proof
    let proof_jstr = match env.new_string(proof_str) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create Java string for proof: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to create Java string for proof: {:?}", e));
            return std::ptr::null_mut();
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
    
    let proof_str = match env.get_string(&proof_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get proof string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get proof string: {:?}", e));
            return 0;
        }
    };
    let proof_str = match proof_str.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert proof to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert proof to Rust string: {:?}", e));
            return 0;
        }
    };
    debug!("Proof string length: {}", proof_str.len());

    let vk_str = match env.get_string(&vk_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get verification key string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get verification key string: {:?}", e));
            return 0;
        }
    };
    let vk_str = match vk_str.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert verification key to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert verification key to Rust string: {:?}", e));
            return 0;
        }
    };
    debug!("Verification key length: {}", vk_str.len());

    let proof = match hex::decode(proof_str) {
        Ok(p) => {
            debug!("Successfully decoded proof, size: {} bytes", p.len());
            p
        },
        Err(e) => {
            error!("Failed to decode proof: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to decode proof: {:?}", e));
            return 0;
        }
    };
    
    let verification_key = match hex::decode(vk_str) {
        Ok(vk) => {
            debug!("Successfully decoded verification key, size: {} bytes", vk.len());
            vk
        },
        Err(e) => {
            error!("Failed to decode verification key: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to decode verification key: {:?}", e));
            return 0;
        }
    };

    let proof_type = match env.get_string(&proof_type_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get proof type string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get proof type string: {:?}", e));
            return 0;
        }
    };
    let proof_type = match proof_type.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert proof type to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert proof type to Rust string: {:?}", e));
            return 0;
        }
    };
    info!("Using proof type: {}", proof_type);

    let verdict = if proof_type == "ultra_honk" {
        match verify_ultra_honk(proof, verification_key) {
            Ok(v) => {
                info!("Verification complete, result: {}", v);
                v
            },  
            Err(e) => {
                error!("Verification failed with error: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Verification failed: {:?}", e));
                return 0;
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
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Verification failed: {:?}", e));
                return 0;
            }
        }
    } else {
        error!("Unsupported proof type: {}", proof_type);
        throw_exception(&mut env, "java/lang/IllegalArgumentException", "Ultra honk and Ultra honk keccak are the only proof types supported for now");
        return 0;
    };

    jboolean::from(verdict)
}

#[no_mangle]
pub extern "system" fn Java_com_noirandroid_lib_Noir_00024Companion_get_1verification_1key<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    circuit_bytecode_jstr: JString<'local>,
    proof_type_jstr: JString<'local>,
    low_memory_mode: jboolean
) -> jobject {
    init_logger();
    info!("Getting verification key");
    
    let use_low_memory = low_memory_mode != 0;
    debug!("Low memory mode: {}", use_low_memory);
    
    let circuit_bytecode = match env.get_string(&circuit_bytecode_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get bytecode string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get bytecode string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    let circuit_bytecode = match circuit_bytecode.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert bytecode to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert bytecode to Rust string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    debug!("Circuit bytecode length: {}", circuit_bytecode.len());

    let proof_type = match env.get_string(&proof_type_jstr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get proof type string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get proof type string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    let proof_type = match proof_type.to_str() {
        Ok(s) => s.to_owned(),
        Err(e) => {
            error!("Failed to convert proof type to Rust string: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to convert proof type to Rust string: {:?}", e));
            return std::ptr::null_mut();
        }
    };
    info!("Using proof type: {}", proof_type);

    let vk = if proof_type == "ultra_honk" {
        match get_ultra_honk_verification_key(&circuit_bytecode, use_low_memory) {
            Ok(key) => {
                info!("Successfully retrieved verification key, size: {} bytes", key.len());
                key
            },
            Err(e) => {
                error!("Failed to get verification key: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get verification key: {:?}", e));
                return std::ptr::null_mut();
            }
        }
    } else if proof_type == "ultra_honk_keccak" {
        match get_ultra_honk_keccak_verification_key(&circuit_bytecode, false, use_low_memory) {
            Ok(key) => {
                info!("Successfully retrieved verification key, size: {} bytes", key.len());
                key
            },
            Err(e) => {
                error!("Failed to get verification key: {:?}", e);
                throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to get verification key: {:?}", e));
                return std::ptr::null_mut();
            }
        }
    } else {
        error!("Unsupported proof type: {}", proof_type);
        throw_exception(&mut env, "java/lang/IllegalArgumentException", "Ultra honk and Ultra honk keccak are the only proof types supported for now");
        return std::ptr::null_mut();
    };

    let vk_str = hex::encode(&vk);
    debug!("Encoded verification key length: {}", vk_str.len());

    // Create and return a Java string containing the proof
    let vk_jstr = match env.new_string(vk_str) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create Java string for verification key: {:?}", e);
            throw_exception(&mut env, "java/lang/RuntimeException", &format!("Failed to create Java string for vk: {:?}", e));
            return std::ptr::null_mut();
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
