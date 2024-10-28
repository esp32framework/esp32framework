extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Path;

/// Imports both rust test crate and the received module. Then creates a struct `EspTest<'a>` that contains a 
/// `&'a TestDescAndFn` and implements the Esp32Test trait for it.
fn get_test_desc_and_fn_impl(test_module_path: Path)->TokenStream2{
    quote! {
        extern crate test;
        use #test_module_path::*;

        struct EspTest<'a>{
            inner: &'a test::TestDescAndFn
        }

        impl<'a> Esp32Test for EspTest<'a> {
            fn execute(&self) -> Result<(), TestExecutionFailures> {
                match self.inner.testfn {
                    test::TestFn::StaticTestFn(func) => func().map_err(|_| TestExecutionFailures::TestFailed),
                    test::TestFn::DynTestFn(_) => Err(TestExecutionFailures::DynamicTestNotSupported),
                    _ => Err(TestExecutionFailures::BenchTestNotSupported),
                }
            }

            fn name(&self)-> &str{
                self.inner.desc.name.as_slice()
            }
        }
    }
}

/// Creates a function `esp_test_runner` that maps the `&[&TestDescAndFn]` into `&[EspTest]` and executes 
/// esp32_test_runner
fn get_test_runner()->TokenStream2{
    quote!{
        pub fn esp_test_runner(tests: &[&test::TestDescAndFn]){
            esp32_test_runner(&tests.into_iter().map(|t| EspTest{inner:t}).collect::<Vec<EspTest>>())
        }
    }
}

#[proc_macro]
/// This macro creates all of the auxiliary code that needs to be included in the crate, in order to be 
/// able to set the custom test_runner.
/// 
/// # Arguments
/// This macro receives one argument, which must be the path towards the esp32framework test module
/// 
/// # Example
/// 
/// As the first lines of your lib or main file include
/// 
/// ```
/// #![feature(custom_test_frameworks)]
/// #![feature(test)]
/// #![test_runner(test_runner_mod::esp33_test_runner)]
/// esp32_testing_macro::use_esp32_tests!(esp32framework::esp_test);
/// ```
/// 
/// This will generate the following code
/// 
/// ```
/// #![feature(custom_test_frameworks)]
/// #![feature(test)]
/// #![test_runner(test_runner_mod::esp_test_runner)]
/// #[cfg(test)]
/// mod test_runner_mod{
///    extern crate test;
///    use esp32framework::esp_test::*;
/// 
///    struct EspTest<'a>{
///        inner: &'a test::TestDescAndFn
///    }
///    
///    impl<'a> Esp32Test for EspTest<'a> {
///        fn execute(&self) -> Result<(), TestExecutionFailures> {
///            match self.inner.testfn {
///                test::TestFn::StaticTestFn(func) => func().map_err(|_| TestExecutionFailures::TestFailed),
///                test::TestFn::DynTestFn(_) => Err(TestExecutionFailures::DynamicTestNotSupported),
///                _ => Err(TestExecutionFailures::BenchTestNotSupported),
///            }
///        }
///    
///        fn name(&self)-> &str{
///            self.inner.desc.name.as_slice()
///        }
///    }
///    pub fn esp_test_runner(tests: &[&test::TestDescAndFn]){
///        esp32_test_runner(&tests.into_iter().map(|t| EspTest{inner:t}).collect::<Vec<EspTest>>())
///    }
/// }
/// ```
pub fn use_esp32_tests(input: TokenStream)-> TokenStream{
    let test_module_path= syn::parse_macro_input!(input as Path);
    let esp32_test = get_test_desc_and_fn_impl(test_module_path);
    let test_runner = get_test_runner();

    let output = quote! {
        #[cfg(test)]
        mod test_runner_mod{
            #esp32_test
            #test_runner
        }
    };
    TokenStream::from(output)
}