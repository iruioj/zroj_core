use syn::LitInt;

extern crate proc_macro;

#[proc_macro]
pub fn time(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: LitInt = syn::parse_macro_input!(input);
    let digits: u64 = input.base10_digits().parse().unwrap();
    let suf = input.suffix();

    let val = match suf {
        "ms" => digits,
        "s" => digits * 1000,
        "min" => digits * 60 * 1000,
        "h" => digits * 60 * 60 * 1000,
        _ => panic!("invalid literal suffix"),
    };

    let mut output = proc_macro2::TokenStream::new();

    output.extend(quote::quote!( Elapse::from(#val) ));

    output.into()
}

#[proc_macro]
pub fn mem(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: LitInt = syn::parse_macro_input!(input);
    let digits: u64 = input.base10_digits().parse().unwrap();
    let suf = input.suffix();

    let val = match suf {
        "b" => digits,
        "k" => digits << 10,
        "kb" => digits << 10,
        "m" => digits << 20,
        "mb" => digits << 20,
        "g" => digits << 30,
        "gb" => digits << 30,
        _ => panic!("invalid literal suffix"),
    };

    let mut output = proc_macro2::TokenStream::new();

    output.extend(quote::quote!( Memory::from(#val) ));

    output.into()
}
