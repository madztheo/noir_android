package com.noirandroid.testapp

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.noirandroid.lib.Noir
import com.noirandroid.lib.Circuit
import java.math.BigInteger
import android.util.Log
class MainActivity : AppCompatActivity() {
    private val testCircuitJson = """{"noir_version":"1.0.0-beta.7+913ee6308f6ea040608df452a66bcb20bece3ca6","hash":"7616458007365582936","abi":{"parameters":[{"name":"a","type":{"kind":"field"},"visibility":"private"},{"name":"b","type":{"kind":"field"},"visibility":"private"},{"name":"result","type":{"kind":"field"},"visibility":"public"}],"return_type":null,"error_types":{}},"bytecode":"H4sIAAAAAAAA/62QQQqAMAwErfigpEna5OZXLLb/f4KKLZbiTQdCQg7Dsm66mc9x00O717rhG9ico5cgMOfoMxJu4C2pAEsKioqisnslysoaLVkEQ6aMRYxKFc//ZYQr29L10XfhXv4jB52E+OpMAQAA","debug_symbols":"lZDBCoMwDIbfJeceZKADX2UMqTVKIaQltoMhvvuirJsedtgpTf5+f8i/wIB9njrPY5ihvS3QiyfyU0fB2eQD63RZDZS2S4KoIzjoSkUryAlazkQGHpby/mmOlvearKhaGUAetKrh6Am312q+dPUbbQrb1B+4/p++num7dtZ5OV0LFbSXdTMTb3vCdwJjZncIJD1jUUpkUYLDIQtudrumC14=","file_map":{"50":{"source":"fn main(a: Field, b: Field, result: pub Field) {\n    assert(a * b == result);\n}\n\n#[test]\nfn test_main() {\n    main(2, 5, 10);\n}\n","path":"/Users/madztheo/Documents/Ocelots/libs/noir_rs/circuits/crates/product/src/main.nr"}},"names":["main"],"brillig_names":[]}""".trimIndent()
    

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        
        val statusTextView = findViewById<TextView>(R.id.status_text)
        
        try {
            val circuit = Circuit.fromJsonManifest(testCircuitJson, 40)
            val witness = circuit.execute(mapOf("a" to "0x2", "b" to "0x3", "result" to "0x6"))
            val a = witness[0].last()
            val b = witness[1].last()
            val result = witness[2].last()
            var result_str = "Circuit Execution Result:\n"
            result_str += "a: $a\n"
            result_str += "b: $b\n"
            result_str += "result: $result\n"
            circuit.setupSrs()
            val vkey = circuit.getVerificationKey()
            val proof = circuit.prove(mapOf("a" to "0x2", "b" to "0x3", "result" to "0x6"), vkey, "ultra_honk")
            // Truncate the proof to 100 characters
            val truncatedProof = proof.substring(0, Math.min(proof.length, 100))
            result_str += "Proof: $truncatedProof\n"
            // Truncate the verification key to 100 characters
            val truncatedVkey = vkey.substring(0, Math.min(vkey.length, 100))
            result_str += "Verification Key: $truncatedVkey\n"
            val verified = circuit.verify(proof, vkey, "ultra_honk")
            result_str += "Verified: $verified\n"
            statusTextView.text = result_str
        } catch (e: Exception) {
            statusTextView.text = "JNI Test Failed: ${e.message}"
            e.printStackTrace()
        }
    }
} 