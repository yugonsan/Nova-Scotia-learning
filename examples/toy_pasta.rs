use std::{collections::HashMap, env::current_dir, time::Instant};

use nova_scotia::{
    circom::reader::load_r1cs, create_public_params, create_recursive_circuit, FileLocation, F, S,
};
use nova_snark::{
    provider,
    traits::{circuit::StepCircuit, Group},
    CompressedSNARK, PublicParams,
};
use serde_json::json;

fn run_test(circuit_filepath: String, witness_gen_filepath: String) {
    // The cycle of curves we use, can be any cycle supported by Nova
    type G1 = pasta_curves::pallas::Point;
    type G2 = pasta_curves::vesta::Point;

    // 
    println!(
        "Running test with witness generator: {} and group: {}",
        witness_gen_filepath,
        std::any::type_name::<G1>()
    );
    // interation means the step of the folding.
    let iteration_count = 1000;
    //  current directory path
    let root = current_dir().unwrap();

    // circuit file means the .r1cs file 
    let circuit_file = root.join(circuit_filepath);
    // loads R1CS file into memory
    let r1cs = load_r1cs::<G1, G2>(&FileLocation::PathBuf(circuit_file));

    // witness generation file means the .wasm file for generating witness
    let witness_generator_file = root.join(witness_gen_filepath);

    // ベクトルの要素がHashmap（key, value）
    let mut private_inputs = Vec::new();
    // iteration回数までのprivate inputの作成を繰り返します。
    for i in 0..iteration_count {
        let mut private_input = HashMap::new();
        // Hashmapのkeyにstringの"adder"で、valueにi番目の数字だが、これはjson表現
        private_input.insert("adder".to_string(), json!(i));
        // Hashmapをprivate_inputsベクトルに追加
        private_inputs.push(private_input);
    }

    // 初めのstep_in[2]の値を入れます。iterationでいう0回目のfolding
    let start_public_input = [F::<G1>::from(1000), F::<G1>::from(1000)];

    // Then, create the public parameters (CRS) using the create_public_params function:
    let pp: PublicParams<G1, G2, _, _> = create_public_params(r1cs.clone());

    println!(
        "Number of constraints per step (primary circuit): {}",
        pp.num_constraints().0
    );
    println!(
        "Number of constraints per step (secondary circuit): {}",
        pp.num_constraints().1
    );

    println!(
        "Number of variables per step (primary circuit): {}",
        pp.num_variables().0
    );
    println!(
        "Number of variables per step (secondary circuit): {}",
        pp.num_variables().1
    );

    println!("Creating a RecursiveSNARK...");
    let start = Instant::now();

    // foldingを行う
    let recursive_snark = create_recursive_circuit(
        // witness gen file
        FileLocation::PathBuf(witness_generator_file),
        // r1cs circuit
        r1cs,
        // private inputs ベクトル
        private_inputs,
        // 初めのpublic input
        start_public_input.to_vec(),
        // public parameters
        &pp,
    )
    .unwrap();
    println!("RecursiveSNARK creation took {:?}", start.elapsed());

    // TODO: empty?
    let z0_secondary = [F::<G2>::from(0)];

    // 多分foldingができているかの検証
    // verify the recursive SNARK
    println!("Verifying a RecursiveSNARK...");
    let start = Instant::now();
    let res = recursive_snark.verify(&pp, iteration_count, &start_public_input, &z0_secondary);
    println!(
        "RecursiveSNARK::verify: {:?}, took {:?}",
        res,
        start.elapsed()
    );
    assert!(res.is_ok());

    // produce a compressed SNARK
    println!("Generating a CompressedSNARK using Spartan with IPA-PC...");
    let start = Instant::now();
    // ppを利用してpkとvkを作成します
    let (pk, vk) = CompressedSNARK::<_, _, _, _, S<G1>, S<G2>>::setup(&pp).unwrap();
    // ppとpkとrecursive snark(folding)を証明します
    let res = CompressedSNARK::<_, _, _, _, S<G1>, S<G2>>::prove(&pp, &pk, &recursive_snark);
    println!(
        "CompressedSNARK::prove: {:?}, took {:?}",
        res.is_ok(),
        start.elapsed()
    );
    assert!(res.is_ok());
    let compressed_snark = res.unwrap();

    // verify the compressed SNARK
    println!("Verifying a CompressedSNARK...");
    let start = Instant::now();
    // proofのverifyをやります。
    let res = compressed_snark.verify(
        &vk,
        iteration_count,
        start_public_input.to_vec(),
        z0_secondary.to_vec(),
    );
    println!(
        "CompressedSNARK::verify: {:?}, took {:?}",
        res.is_ok(),
        start.elapsed()
    );
    assert!(res.is_ok());
}

fn main() {
    let group_name = "pasta";

    let circuit_filepath = format!("examples/toy/{}/toy.r1cs", group_name);
    for witness_gen_filepath in [
        // format!("examples/toy/{}/toy_cpp/toy", group_name),
        format!("examples/toy/{}/toy_js/toy.wasm", group_name),
    ] {
        run_test(circuit_filepath.clone(), witness_gen_filepath);
    }
}
