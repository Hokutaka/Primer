use primer_lang::run_vm;

#[test]
fn hello_example_adds_two_named_integers() {
    let output = run_vm(include_str!("../examples/hello.prim")).unwrap();

    assert_eq!(output, "42\n");
}

#[test]
fn floating_point_example_exposes_precision_and_inferred_type() {
    let output = run_vm(include_str!("../examples/floating_point.prim")).unwrap();

    assert_eq!(
        output,
        "0.300000012\n0.30000000000000004\n0.30000000000000004\n"
    );
}

#[test]
fn coin_change_finds_a_better_answer_than_greedy_selection() {
    let output = run_vm(include_str!("../examples/coin_change.prim")).unwrap();

    assert_eq!(output, "1\n2\n1\n1\n2\n2\n3\n3\n");
}

#[test]
fn shortest_paths_preserves_unreachable_pairs_and_finds_indirect_routes() {
    let output = run_vm(include_str!("../examples/shortest_paths.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "-1\n11\n6\n6\n",
            "0\n3\n5\n6\n",
            "-1\n0\n2\n3\n",
            "-1\n-1\n0\n1\n",
            "-1\n-1\n-1\n0\n",
        )
    );
}

#[test]
fn heat_diffusion_updates_from_one_time_step_and_preserves_initial_values() {
    let output = run_vm(include_str!("../examples/heat_diffusion.prim")).unwrap();
    let values: Vec<f64> = output.lines().map(|line| line.parse().unwrap()).collect();

    assert_eq!(
        values,
        [
            0.0, 4.0, 8.0, 4.0, 0.0, 0.0, 4.0, 6.0, 4.0, 0.0, 0.0, 3.5, 5.0, 3.5, 0.0, 0.0, 3.0,
            4.25, 3.0, 0.0, 16.0,
        ]
    );
}

#[test]
fn linear_regression_learns_the_line_and_reduces_loss() {
    let output = run_vm(include_str!("../examples/linear_regression.prim")).unwrap();
    let values: Vec<f64> = output.lines().map(|line| line.parse().unwrap()).collect();

    assert_eq!(values.len(), 26);
    assert!(values.iter().all(|value| value.is_finite()));
    assert_eq!(values[0], 9.0);
    let mut previous_loss = values[0];
    for (index, checkpoint) in values[1..25].as_chunks::<4>().0.iter().enumerate() {
        assert_eq!(checkpoint[0], ((index + 1) * 10) as f64);
        assert!(checkpoint[3] >= 0.0);
        assert!(checkpoint[3] < previous_loss);
        previous_loss = checkpoint[3];
    }
    // 正解の直線は y = 2x + 1。文字列の丸め方ではなく収束を検証します。
    assert!((values[22] - 2.0).abs() < 0.00001);
    assert!((values[23] - 1.0).abs() < 0.00001);
    assert!(values[24] < 0.0000000001);
    assert!((values[25] - 7.0).abs() < 0.00001);
}

#[test]
fn integer_limits_example_runs_without_overflow() {
    let output = run_vm(include_str!("../examples/integer_limits.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "-9223372036854775808\n",
            "9223372036854775807\n",
            "-9223372036854775807\n",
            "9223372036854775806\n",
            "false\n",
        )
    );
}

#[test]
fn square_root_example_runs() {
    let output = run_vm(include_str!("../examples/square_root.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "1.5\n",
            "1.41666666666666652\n",
            "1.41421568627450966\n",
            "1.41421356237468987\n",
            "1.41421356237309492\n",
        )
    );
}

#[test]
fn logistic_map_example_exposes_f32_and_f64_divergence() {
    let output = run_vm(include_str!("../examples/logistic_map.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "0.624000013\n",
            "0.62400000000000011\n",
            "0.915033638\n",
            "0.91503359999999978\n",
            "0.303213596\n",
            "0.30321373239705673\n",
            "0.82397294\n",
            "0.82397314304332092\n",
            "0.565661967\n",
            "0.56566147008786449\n",
            "0.958185136\n",
            "0.95818542824901176\n",
        )
    );
}

#[test]
fn boolean_comparisons_example_runs() {
    let output = run_vm(include_str!("../examples/boolean_comparisons.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "true\n", "false\n", "true\n", "true\n", "true\n", "true\n", "true\n", "true\n",
            "true\n",
        )
    );
}

#[test]
fn conditional_example_runs() {
    let output = run_vm(include_str!("../examples/conditional.prim")).unwrap();

    assert_eq!(output, "true\n42\n");
}

#[test]
fn while_square_root_example_runs() {
    let output = run_vm(include_str!("../examples/while_square_root.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "1.5\n",
            "1.41666666666666652\n",
            "1.41421568627450966\n",
            "1.41421356237468987\n",
            "1.41421356237309492\n",
        )
    );
}

#[test]
fn loop_control_example_runs() {
    let output = run_vm(include_str!("../examples/loop_control.prim")).unwrap();

    assert_eq!(output, "12\n6\n");
}

#[test]
fn for_sum_example_runs() {
    let output = run_vm(include_str!("../examples/for_sum.prim")).unwrap();

    assert_eq!(output, "14\n");
}

#[test]
fn euclidean_gcd_example_runs() {
    let output = run_vm(include_str!("../examples/euclidean_gcd.prim")).unwrap();

    assert_eq!(output, "21\n");
}

#[test]
fn fibonacci_example_runs() {
    let output = run_vm(include_str!("../examples/fibonacci.prim")).unwrap();

    assert_eq!(output, "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n");
}

#[test]
fn factorial_example_runs() {
    let output = run_vm(include_str!("../examples/factorial.prim")).unwrap();

    assert_eq!(output, "3628800\n");
}

#[test]
fn collatz_example_runs() {
    let output = run_vm(include_str!("../examples/collatz.prim")).unwrap();

    assert_eq!(output, "111\n9232\n");
}

#[test]
fn prime_check_example_runs() {
    let output = run_vm(include_str!("../examples/prime_check.prim")).unwrap();

    assert_eq!(output, "true\n");
}

#[test]
fn integer_square_root_example_runs() {
    let output = run_vm(include_str!("../examples/integer_square_root.prim")).unwrap();

    assert_eq!(output, "1414\n");
}

#[test]
fn exponentiation_by_squaring_example_runs() {
    let output = run_vm(include_str!("../examples/exponentiation_by_squaring.prim")).unwrap();

    assert_eq!(output, "1594323\n");
}

#[test]
fn pythagorean_triples_example_runs() {
    let output = run_vm(include_str!("../examples/pythagorean_triples.prim")).unwrap();

    assert_eq!(output, "3\n4\n5\n5\n12\n13\n6\n8\n10\n9\n12\n15\n");
}

#[test]
fn product_point_example_runs() {
    let output = run_vm(include_str!("../examples/product-point.prim")).unwrap();

    assert_eq!(output, "0\n2\n");
}

#[test]
fn functions_example_runs() {
    let output = run_vm(include_str!("../examples/functions.prim")).unwrap();

    assert_eq!(output, "42\n");
}

#[test]
fn function_values_example_runs() {
    let output = run_vm(include_str!("../examples/function_values.prim")).unwrap();

    assert_eq!(output, "10\n15\n20\n1\n2\n");
}

#[test]
fn fixed_arrays_example_runs() {
    let output = run_vm(include_str!("../examples/fixed_arrays.prim")).unwrap();

    assert_eq!(output, "108\n4\n");
}

#[test]
fn bubble_sort_example_runs() {
    let output = run_vm(include_str!("../examples/bubble_sort.prim")).unwrap();

    assert_eq!(output, "1\n2\n3\n4\n5\n8\n");
}

#[test]
fn product_arrays_example_runs() {
    let output = run_vm(include_str!("../examples/product_arrays.prim")).unwrap();

    assert_eq!(output, "3\n6\n2\n2\n100\n");
}

#[test]
fn matrix_vector_product_example_runs() {
    let output = run_vm(include_str!("../examples/matrix_vector_product.prim")).unwrap();

    assert_eq!(output, "14\n32\n50\n");
}

#[test]
fn matrix_composition_example_runs() {
    let output = run_vm(include_str!("../examples/matrix_composition.prim")).unwrap();

    assert_eq!(output, "0\n3\n2\n0\n4\n5\n15\n8\n");
}

#[test]
fn xor_neural_network_example_runs() {
    let output = run_vm(include_str!("../examples/xor_neural_network.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "0\n", "0\n", "0\n", "true\n", "0\n", "1\n", "1\n", "true\n", "1\n", "0\n", "1\n",
            "true\n", "1\n", "1\n", "0\n", "true\n",
        )
    );
}
