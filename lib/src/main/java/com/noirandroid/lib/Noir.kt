package com.noirandroid.lib

class Noir {
    companion object {
        external fun prove(circuitBytecode: String, initialWitness: Map<String, String>, proofType: String, numPoints: String): Proof

        external fun verify(circuitBytecode: String, proof: Proof, proofType: String, numPoints: String): Boolean

        external fun setup_srs(circuitBytecode: String, srsPath: String?): Int
    }
}