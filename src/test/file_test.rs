use crate::jit::JIT;

macro_rules! file_test {
    ($name: ident) => {
        #[test]
        fn $name() {
            let mut jit = JIT::new(false, false, false).unwrap();

            let contents = include_str!(concat!("test_data/", stringify!($name), ".epi"));
            match jit.compile("test_input", &contents.trim()) {
                Ok(func) => {
                    let result = unsafe { func.call() };
                    assert_eq!(
                        result,
                        include!(concat!("test_data/", stringify!($name), ".ans"))
                    );
                }
                Err(err) => assert!(false, format!("{}", err)),
            }
        }
    };
}

file_test!(curry);
