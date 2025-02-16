package com.noirandroid.lib

class Noir {
    companion object {
        external fun prove(circuitBytecode: String, initialWitness: Map<String, String>, proofType: String?, recursive: String?): Proof

        external fun verify(proof: Proof, proofType: String?): Boolean

        external fun setup_srs(circuitBytecode: String, srsPath: String?, recursive: String?): Int
    }
}