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
        
        external fun prove(circuitBytecode: String, initialWitness: Map<String, String>, proofType: String?, recursive: String?): Proof

        external fun verify(proof: Proof, proofType: String?): Boolean

        external fun setup_srs(circuitBytecode: String, srsPath: String?, recursive: String?): Int
        
        external fun execute(circuitBytecode: String, initialWitness: Map<String, String>): Array<String>
    }
}