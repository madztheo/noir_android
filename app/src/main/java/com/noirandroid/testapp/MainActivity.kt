package com.noirandroid.testapp

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.noirandroid.lib.Noir
import com.noirandroid.lib.Circuit
import java.math.BigInteger
import android.util.Log
class MainActivity : AppCompatActivity() {
    private val testCircuitJson = """{"noir_version":"1.0.0-beta.3+7aa23ec674b2877745595b1584ade4733abeac71","hash":12594608413049942367,"abi":{"parameters":[{"name":"a","type":{"kind":"field"},"visibility":"private"},{"name":"b","type":{"kind":"field"},"visibility":"private"},{"name":"result","type":{"kind":"field"},"visibility":"public"}],"return_type":null,"error_types":{}},"bytecode":"H4sIAAAAAAAA/62QQQqAMAwErfigpEna5OZXLLb/f4KKLZbiTQdCQg7Dsm66mc9x00O717rhG9ico5cgMOfoMxJu4C2pAEsKioqisnslysoaLVkEQ6aMRYxKFc//ZYQr29L10XfhXv4jB52E+OpMAQAA","debug_symbols":"TYxLCsMwDAXvonUW6aIt+CqlBH/kIDCyke1CMbl7ldBAlvOGNwMCur4uxDFXMK8BKXvbKLPSgPmYarG8U21WGpjHPAFyAPO8bxNESqjbbXsrOKGUaF2uEZ0/Vsi6hH+Mnf3Ftm85zfkvkj2GLriXDqf5Hw==","file_map":{},"names":["main"],"brillig_names":[]}""".trimIndent()
    

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        
        val statusTextView = findViewById<TextView>(R.id.status_text)
        
        try {
            val circuit = Circuit.fromJsonManifest(testCircuitJson)
            val witness = circuit.execute(mapOf("a" to "0x2", "b" to "0x3", "result" to "0x6"))
            val a = witness[0].last()
            val b = witness[1].last()
            val result = witness[2].last()
            var result_str = "Circuit Execution Result:\n"
            result_str += "a: $a\n"
            result_str += "b: $b\n"
            result_str += "result: $result\n"
            statusTextView.text = result_str
        } catch (e: Exception) {
            statusTextView.text = "JNI Test Failed: ${e.message}"
            e.printStackTrace()
        }
    }
} 