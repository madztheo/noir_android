package com.noirandroid.lib

class Noir {
    companion object {
        // Static initializer block to ensure library is loaded when class is first accessed
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
        
        external fun prove(circuitBytecode: String, initialWitness: Map<String, String>, vk: String, proofType: String?, lowMemoryMode: Boolean): String

        external fun verify(proof: String, vk: String, proofType: String?): Boolean

        external fun setup_srs(size: Int, srsPath: String?): Int

        external fun setup_srs_from_bytecode(circuitBytecode: String, srsPath: String?): Int
        
        external fun execute(circuitBytecode: String, initialWitness: Map<String, String>): Array<String>

        external fun get_verification_key(circuitBytecode: String, proofType: String?, lowMemoryMode: Boolean): String
    }
}