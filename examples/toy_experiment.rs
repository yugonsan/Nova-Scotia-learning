use std::{collections::HashMap, env::current_dir, time::Instant, path::{Path, PathBuf}};

use ff::PrimeField;
use itertools::Itertools;
use nova_scotia::{
    circom::reader::generate_witness_from_bin, circom::reader::load_r1cs, create_public_params, create_recursive_circuit, FileLocation, F, S,
};
use nova_snark::{
    traits::Group,
};
use serde_json::json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use num_bigint::BigInt;
use num_traits::Num;

#[cfg(not(target_family = "wasm"))]
use nova_scotia::circom::reader::generate_witness_from_wasm;

#[cfg(target_family = "wasm")]
use nova_scotia::circom::wasm::generate_witness_from_wasm;



fn print_r1cs<Fr: PrimeField>(r1cs: &nova_scotia::circom::circuit::R1CS<Fr>) {
    println!("num_inputs: {}", r1cs.num_inputs);
    println!("num_aux: {}", r1cs.num_aux);
    println!("num_variables: {}", r1cs.num_variables);

    println!("constraints:");
    for (a, b, c) in &r1cs.constraints {
        print!("A:");
        for (idx, coeff) in a {
            print!(" (idx: {}, coeff: {:?})", idx, coeff); 
        }
        println!();
        
        print!("B:");
        for (idx, coeff) in b {
            print!(" (idx: {}, coeff: {:?})", idx, coeff); 
        }
        println!();

        print!("C:");
        for (idx, coeff) in c {
            print!(" (idx: {}, coeff: {:?})", idx, coeff); 
        }
        println!();
    }
}


fn print_r1cs_matrix<Fr: PrimeField>(r1cs: &nova_scotia::circom::circuit::R1CS<Fr>) {
    if r1cs.constraints.is_empty() {
        panic!("r1cs.constraints is empty!");
    }

    // 最大のidxの値を取得
    let m = r1cs.constraints.iter().flat_map(|(a, b, c)| {
        a.iter().chain(b.iter()).chain(c.iter()).map(|(idx, _)| *idx)
    }).max().unwrap_or(0) + 1;

    let n = r1cs.constraints.len(); // 制約の数

    // 初期化
    let mut matrix_a: Vec<Vec<Fr>> = vec![vec![Fr::from(0); m]; n];
    let mut matrix_b: Vec<Vec<Fr>> = vec![vec![Fr::from(0); m]; n];
    let mut matrix_c: Vec<Vec<Fr>> = vec![vec![Fr::from(0); m]; n];

    // 行列の構築
    for (i, (a, b, c)) in r1cs.constraints.iter().enumerate() {
        for (idx, coeff) in a {
            matrix_a[i][*idx] = *coeff;
        }
        for (idx, coeff) in b {
            matrix_b[i][*idx] = *coeff;
        }
        for (idx, coeff) in c {
            matrix_c[i][*idx] = *coeff;
        }
    }

    // 行列の表示
    println!("Matrix A:");
    for row in &matrix_a {
        println!("{:?}", row);
    }

    println!("\nMatrix B:");
    for row in &matrix_b {
        println!("{:?}", row);
    }

    println!("\nMatrix C:");
    for row in &matrix_c {
        println!("{:?}", row);
    }
}


#[derive(Serialize, Deserialize)]
struct CircomInput {
    step_in: Vec<String>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

fn compute_witness_vector<G1, G2>(
    current_public_input: Vec<String>,
    private_input: HashMap<String, Value>,
    witness_generator_file: FileLocation,
) -> Vec<<G1 as Group>::Scalar>
where
    G1: Group<Base = <G2 as Group>::Scalar>,
    G2: Group<Base = <G1 as Group>::Scalar>,
{
    let decimal_stringified_input: Vec<String> = current_public_input
        .iter()
        .map(|x| BigInt::from_str_radix(x, 16).unwrap().to_str_radix(10))
        .collect();

    let input = CircomInput {
        step_in: decimal_stringified_input.clone(),
        extra: private_input.clone(),
    };

    let is_wasm = match &witness_generator_file {
        FileLocation::PathBuf(path) => path.extension().unwrap_or_default() == "wasm",
        FileLocation::URL(_) => true,
    };
    let input_json = serde_json::to_string(&input).unwrap();

    if is_wasm {
        generate_witness_from_wasm::<F<G1>>(
            &witness_generator_file,
            &input_json,
            &Path::new("path_to_witness_output"),
        )
    } else {
        let witness_generator_file = match &witness_generator_file {
            FileLocation::PathBuf(path) => path,
            FileLocation::URL(_) => panic!("unreachable"),
        };
        generate_witness_from_bin::<F<G1>>(
            &witness_generator_file,
            &input_json,
            &Path::new("path_to_witness_output"),
        )
    }
}

fn main() {
    let group_name = "bn254";
    let circuit_filepath = format!("examples/toy/{}/toy.r1cs", group_name);

    let root = std::env::current_dir().unwrap();
    // The cycle of curves we use, can be any cycle supported by Nova
    type G1 = pasta_curves::pallas::Point;
    type G2 = pasta_curves::vesta::Point;

    let circuit_file = root.join(circuit_filepath);

    let r1cs = load_r1cs::<G1, G2>(&FileLocation::PathBuf(circuit_file));

    print_r1cs(&r1cs);
    print_r1cs_matrix(&r1cs);

    let current_public_input = [F::<G1>::from(2)];

    let current_public_input_hex = current_public_input
        .iter()
        .map(|&x| format!("{:?}", x).strip_prefix("0x").unwrap().to_string())
        .collect::<Vec<String>>();
    let current_public_inputs = current_public_input_hex.clone();

    // let mut private_inputs = Vec::new();
    let mut private_input = HashMap::new();
    private_input.insert("adder".to_string(), json!(3));
    // private_inputs.push(private_input);
    let witness_gen_filepath = format!("examples/toy/{}/toy_js/toy.wasm", group_name);
    let witness_generator_file = FileLocation::PathBuf(root.join(witness_gen_filepath));

    let witness_vector = compute_witness_vector::<G1, G2>(current_public_inputs, private_input, witness_generator_file);
    println!("Witness Vector: {:?}", witness_vector);
}

