# Noir Android

## Description

This library lets you generate and verify proofs with Noir on Android.

## Installation

In your `settings.gradle` file or in your root `build.gradle` file, add the following in your `repositories` block:

```gradle
maven { url 'https://jitpack.io' }
```

Then, in your app level `build.gradle` file, add the following in your `dependencies` block:

```gradle
implementation("com.github.madztheo:noir_android:1.0.0-beta.3-1")
```

After this your project should be able to use the library.

## Usage

### Initialize your circuit

To start generating proofs, you first need to initialize your circuit. You can do this by creating a new instance of the `Circuit` class, using the `fromJsonManifest` function and passing it the stringified JSON of the compiled circuit (obtained by running `nargo compile`).

```kotlin
import com.noirandroid.lib.Circuit

val path = "path/to/compiled/circuit.json"
val circuitData = File(path).readText()
val circuit = Circuit.fromJsonManifest(circuitData)
```

### Setup the SRS

Before you can generate proofs, you need to setup the SRS for the circuit. You can do so by calling the `setupSrs` function.

```kotlin
circuit.setupSrs()
```

This will download the SRS from Aztec's server and initialize the SRS for the circuit.
If you want to use a local SRS, you can pass a path to a local SRS file as an argument to the `setupSrs` function.

```kotlin
val srsPath = "path/to/local/srs"
circuit.setupSrs(srsPath)
```

### Generate a proof

To generate a proof, you can call the `prove` method and pass in the inputs for the proof and the proof type. It will return a `Proof` object containing the proof with its public inputs and the verification key.

```kotlin
import com.noirandroid.lib.Proof
import java.util.HashMap
import android.util.Log

val inputs: HashMap<String, Any> = HashMap()
inputs["a"] = 5
inputs["b"] = 3
inputs["result"] = 15

val proof: Proof = circuit.prove(inputs)
Log.d("Proof", proof.proof)
Log.d("Verification key", proof.vk)
```

### Verify a proof

To verify a proof, you can call the `verify` method and pass in the proof object and the proof type. It will return a boolean indicating whether the proof is valid or not.

```kotlin
val isValid = circuit.verify(proof)
```
