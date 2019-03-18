macro_rules! file_test {
    ($name: ident) => {
        #[test]
        fn $name() {
            use crate::codegen::{compile, initialization};
            use inkwell::OptimizationLevel;

            initialization::initialize_native().unwrap();

            let contents = include_str!(concat!("test_data/", stringify!($name), ".epi"));
            match compile::compile_string(&contents.trim(), "test_input") {
                Ok(result) => {
                    let func = result.emit_function(OptimizationLevel::None).unwrap();
                    assert_eq!(
                        unsafe { func.call() },
                        include!(concat!("test_data/", stringify!($name), ".ans"))
                    );
                }
                Err(err) => assert!(false, format!("{}", err)),
            }
        }
    };
}

file_test!(add);
file_test!(shadow);
file_test!(unscoped);
file_test!(ifelse);
file_test!(capture_type);
file_test!(curry);
file_test!(arg_order);
file_test!(infer_poly);
file_test!(capture_list_let);
file_test!(complex_subst_apply);
file_test!(translate_polyfunc_one_candidate);
