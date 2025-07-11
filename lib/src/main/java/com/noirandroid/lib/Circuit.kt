package com.noirandroid.lib

import com.google.gson.Gson
import android.util.Log

data class CircuitManifest(
    val noir_version: String,
    // All JSON numbers are parsed as Double by Gson
    val hash: Double,
    val abi: Abi,
    val bytecode: String,
    val debug_symbols: String,
    val file_map: Map<String, FileMap>,
    val names: List<String>
)

data class Abi(
    val parameters: List<Parameter>,
    val param_witnesses: Map<String, List<Witness>>,
    val return_type: Any?,
    val return_witnesses: List<Any>,
    val error_types: ErrorTypes
)

data class Parameter(
    val name: String,
    val type: Type,
    val visibility: String?
)

data class Type(
    // struct, array, field, integer, string
    val kind: String,
    val path: String?,
    val type: Type?,
    val fields: List<Parameter>?,
    val length: Double?,
    val sign: String?,
    val width: Double?,
)

data class Witness(
    // All JSON numbers are parsed as Double by Gson
    val start: Double,
    val end: Double
)

data class ErrorTypes(
    val error: Any?
)

data class FileMap(
    val source: String,
    val path: String
)

class Circuit(public val bytecode: String, public val manifest: CircuitManifest, public var num_points: Int = 0, public var size: Int = 0) {

    companion object {
        fun fromJsonManifest(jsonManifest: String, size: Int? = null): Circuit {
            val manifest: CircuitManifest = Gson().fromJson(jsonManifest, CircuitManifest::class.java)
            return Circuit(manifest.bytecode, manifest, 0, size ?: 0)
        }
    }

    init {
        try {
            System.loadLibrary("noir_java")
        } catch (e: UnsatisfiedLinkError) {
            // Log the error but don't crash - test classes can handle the error
            System.err.println("Failed to load noir_java native library: ${e.message}")
            e.printStackTrace()
        } catch (e: Exception) {
            System.err.println("Exception during noir_java library loading: ${e.message}")
            e.printStackTrace()
        }
    }

    fun setupSrs(srs_path: String? = null) {
        try {
            if (size > 0) {
                num_points = Noir.setup_srs(size, srs_path)
            } else {
                num_points = Noir.setup_srs_from_bytecode(bytecode, srs_path)
            }
        } catch (e: Throwable) {
            Log.e("Circuit", "Failed to setup SRS: ${e.message}", e)
            throw RuntimeException("SRS setup failed: ${e.message}", e)
        }
    }

    fun execute(initialWitness: Map<String, Any>): Array<String> {
        try {
            val witness = generateWitnessMap(initialWitness, manifest.abi.parameters, 0)
            return Noir.execute(bytecode, witness)
        } catch (e: Throwable) {
            Log.e("Circuit", "Failed to execute circuit: ${e.message}", e)
            throw RuntimeException("Circuit execution failed: ${e.message}", e)
        }
    }

    fun prove(initialWitness: Map<String, Any>, vk: String? = null, proofType: String? = "ultra_honk"): String {
        if (num_points == 0) {
            throw IllegalArgumentException("SRS not set up")
        }
        try {
            val witness = generateWitnessMap(initialWitness, manifest.abi.parameters, 0)
            return Noir.prove(bytecode, witness, vk ?: getVerificationKey(), proofType)
        } catch (e: Throwable) {
            Log.e("Circuit", "Failed to prove circuit: ${e.message}", e)
            throw RuntimeException("Circuit proving failed: ${e.message}", e)
        }
    }

    fun verify(proof: String, vk: String? = null, proofType: String? = "ultra_honk"): Boolean {
        if (num_points == 0) {
            throw IllegalArgumentException("SRS not set up")
        }
        try {
            return Noir.verify(proof, vk ?: getVerificationKey(), proofType)
        } catch (e: Throwable) {
            Log.e("Circuit", "Failed to verify proof: ${e.message}", e)
            throw RuntimeException("Proof verification failed: ${e.message}", e)
        }
    }

    fun getVerificationKey(proofType: String? = "ultra_honk"): String {
        try {
            return Noir.get_verification_key(bytecode, proofType)
        } catch (e: Throwable) {
            Log.e("Circuit", "Failed to get verification key: ${e.message}", e)
            throw RuntimeException("Failed to get verification key: ${e.message}", e)
        }
    }

    private fun flattenMultiDimensionalArray(array: List<Any>, elementType: Type): List<Any> {
        val flattenedArray = mutableListOf<Any>()
        for (element in array) {
            if (element is List<*>) {
                flattenedArray.addAll(flattenMultiDimensionalArray(element as List<Any>, elementType.type!!))
            } else if(elementType.kind == "string" && element is String) {
                val length = elementType.length!!.toInt()
                for (i in 0 until length) {
                    if (i < element.length) {
                        flattenedArray.add(element.get(i).toDouble())
                    } else {
                        // Pad with 0 if the string is shorter than the expected length
                        // Can happen with strings containing the null character for example
                        flattenedArray.add(0.0)
                    }
                }
            } else {
                flattenedArray.add(element)
            }
        }
        return flattenedArray
    }

    private fun computeTotalLengthOfArray(parameter_type: Type): Int {
        when(parameter_type.kind) {
            "array" -> {
                return parameter_type.length!!.toInt() * computeTotalLengthOfArray(parameter_type.type!!)
            }
            "field", "integer" -> {
                return 1
            }
            "string" -> {
                return parameter_type.length!!.toInt()
            }
            "struct" -> {
                return parameter_type.fields!!.map { computeTotalLengthOfArray(it.type) }.sum()
            }
        }
        return 0
    }

    private fun generateWitnessMap(initialWitness: Map<String, Any>, parameters: List<Parameter>, startIndex: Long): HashMap<String, String> {
        val witness = HashMap<String, String>()
        var index = startIndex
        for (parameter in parameters) {
            val value = initialWitness[parameter.name]
            if (value == null) {
                throw IllegalArgumentException("Missing parameter: ${parameter.name}")
            }
            when (parameter.type.kind) {
                "field", "integer" -> {
                    if (value is Double) {
                        if (parameter.type.width != null && parameter.type.width > 64) {
                            throw IllegalArgumentException("Unsupported number size for parameter: ${parameter.name}. Use a hexadecimal string instead for large numbers.")
                        }
                        witness[index.toString()] = "0x${(value.toLong()).toString(16)}"
                        index++
                    } 
                    // Useful to represent very large numbers that don't fit in a Double
                    else if (value is String) {
                        // Check the number is in hexadecimal format
                        if (!value.startsWith("0x")) {
                            throw IllegalArgumentException("Expected hexadecimal number for parameter: ${parameter.name}. Got ${value.javaClass}")
                        }
                        witness[index.toString()] = value
                        index++
                    
                    } else {
                        throw IllegalArgumentException("Expected integer for parameter: ${parameter.name}. Got ${value.javaClass}")
                    }
                }
                "array" -> {
                    if (value is List<*>) {
                        // Flatten the multi-dimensional array (if not multi-dimensional, it will return the same array)
                        var flattenedArray = flattenMultiDimensionalArray(value as List<Any>, parameter.type.type!!)
                        // Compute the expected length of the array
                        var totalLength = computeTotalLengthOfArray(parameter.type)
                        if (flattenedArray.size != totalLength) {
                            throw IllegalArgumentException("Expected array of length ${parameter.type.length} for parameter: ${parameter.name}. Instead got ${flattenedArray.size}")
                        }
                        for (element in flattenedArray) {
                            if (element is Double) {
                                witness[index.toString()] = "0x${(element.toLong()).toString(16)}"
                                index++
                            } else if(element is String) {
                                // Check the number is in hexadecimal format
                                if (!element.startsWith("0x")) {
                                    throw IllegalArgumentException("Expected hexadecimal number for parameter: ${parameter.name}")
                                }
                                witness[index.toString()] = element
                                index++
                            
                            } else {
                                throw IllegalArgumentException("Unexpected array type for parameter: ${parameter.name}. Got ${element.javaClass}")
                            }

                        }
                    } else {
                        throw IllegalArgumentException("Expected array of integers for parameter: ${parameter.name}. Got ${value.javaClass}")
                    }
                }
                "struct" -> {
                    if (value is Map<*, *>) {
                        val struct = value as Map<String, Any>
                        val structWitness = generateWitnessMap(struct, parameter.type.fields!!, index)
                        for ((key, witnessValue) in structWitness) {
                            witness[key] = witnessValue
                            index++
                        }
                    } else {
                        throw IllegalArgumentException("Expected struct for parameter: ${parameter.name}. Got ${value.javaClass}")
                    }
                }
                "string" -> {
                    if (value is String) {
                        // Transform the string into a byte array
                        val array = value.toByteArray()
                        if (array.size != parameter.type.length!!.toInt()) {
                            throw IllegalArgumentException("Expected string of length ${parameter.type.length} for parameter: ${parameter.name}. Instead got ${array.size}")
                        }
                        for (element in array) {
                            witness[index.toString()] = "0x${(element.toLong()).toString(16)}"
                            index++
                        }
                    } else {
                        throw IllegalArgumentException("Expected string for parameter: ${parameter.name}. Got ${value.javaClass}")
                    }
                }
                else -> throw IllegalArgumentException("Unsupported parameter type: ${parameter.type}. Kind: ${parameter.type.kind}")
            }
        }
        return witness
    }
}
