/*!
This create provides a derive macro for the `fixed_width` crate's `FixedWidth` trait by providing
a set of struct field attributes that can be used to more easily derive the trait.

The derive only works on structs. Additionally, this crate uses features that require Rust version 1.30.0+ to run.

# Installing

Start by adding the dependency to your project in `Cargo.toml`:

```toml
fixed_width = "0.4"
fixed_width_derive = "0.4"
```

# Usage

```rust
use serde_derive::Deserialize;
use fixed_width_derive::FixedWidth;
use fixed_width::FixedWidth;

#[derive(FixedWidth, Deserialize)]
struct Person {
    #[fixed_width(range = "0..6")]
    pub name: String,
    #[fixed_width(range = "6..9", pad_with = "0")]
    pub age: usize,
    #[fixed_width(range = "9..11", name = "height_cm", justify = "right")]
    pub height: usize,
    #[serde(skip)]
    pub gender: String,
}
```

The above sample is equivalent to implementing the following with the `fixed_width`
crate alone:

```rust
use fixed_width::{FixedWidth, FieldSet, Justify};

struct Person {
    pub name: String,
    pub age: usize,
    pub height: usize,
}

impl FixedWidth for Person {
    fn fields() -> FieldSet {
        FieldSet::Seq(vec![
            FieldSet::new_field(0..6),
            FieldSet::new_field(6..9).pad_with('0'),
            FieldSet::new_field(9..11).justify(Justify::Right).name("height_cm"),
        ])
    }
}
```

The full set of options you can supply for the attribute annotations are:

### `range = "x..y"`
Required. Range values must be of type `usize`. The byte range of the given field.

### `pad_with = "c"`
Defaults to `' '`. Must be of type `char`. The character to pad to the left or right after the
value of the field has been converted to bytes. For instance, if the width of
the field was 5, and the value is `"foo"`, then a left justified field padded with `a`
results in: `"fooaa"`.

### `justify = "left|right"`
Defaults to `"left"`. Must be of enum type `Justify`. Indicates whether this field should be justified
left or right once it has been converted to bytes.

### `name = "s"`
Defaults to the name of the struct field. Indicates the name of the field. Useful if you wish to deserialize
fixed width data into a HashMap.

### `skip`
Skips the given field.
!*/

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use crate::field_def::{Context, FieldDef};
use proc_macro::TokenStream;
use std::result;
use syn::DeriveInput;

mod field_def;

#[proc_macro_derive(FixedWidth, attributes(fixed_width))]
pub fn fixed_width(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    impl_fixed_width(&input)
}

fn impl_fixed_width(ast: &DeriveInput) -> TokenStream {
    let fields: Vec<syn::Field> = match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                panic!("struct has unnamed fields");
            }
            fields.iter().cloned().collect()
        }
        _ => panic!("#[derive(FixedWidth)] can only be used with structs"),
    };

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .filter(should_skip)
        .map(build_field_def)
        .map(build_fixed_width_field)
        .collect();

    let quote = quote! {
        impl #impl_generics fixed_width::FixedWidth for #ident #ty_generics #where_clause {
            fn fields() -> fixed_width::FieldSet {
                fixed_width::field_seq![#(#tokens),*]
            }
        }
    };

    quote.into()
}

fn should_skip(field: &&syn::Field) -> bool {
    !Context::from_field(*field).skip
}

fn build_field_def(field: &syn::Field) -> FieldDef {
    let ctx = Context::from_field(field);

    let name = match ctx.metadata.get("name") {
        Some(name) => name.value.clone(),
        None => ctx.field_name(),
    };

    let range = if let Some(r) = ctx.metadata.get("range") {
        let range_parts = r
            .value
            .split("..")
            .map(str::parse)
            .filter_map(result::Result::ok)
            .collect::<Vec<usize>>();

        if range_parts.len() != 2 {
            panic!("Invalid range {} for field: {}", r.value, ctx.field_name());
        }

        range_parts[0]..range_parts[1]
    } else {
        panic!("Must supply a byte range for field: {}", ctx.field_name());
    };

    let pad_with = ctx.metadata.get("pad_with").map_or(' ', |c| {
        if c.value.len() != 1 {
            panic!("pad_with must be a char for field: {}", ctx.field_name());
        }

        c.value.chars().nth(0).unwrap()
    });

    let justify = match ctx.metadata.get("justify") {
        Some(j) => match j.value.to_lowercase().trim().as_ref() {
            "left" | "right" => j.value.clone(),
            _ => panic!(
                "justify must be 'left' or 'right' for field: {}",
                ctx.field_name()
            ),
        },
        None => "left".to_string(),
    };

    FieldDef {
        ident: ctx.field.clone().ident.unwrap(),
        field_type: field.ty.clone(),
        name,
        pad_with,
        range,
        justify,
    }
}

fn build_fixed_width_field(field_def: FieldDef) -> proc_macro2::TokenStream {
    let name = field_def.name;
    let start = field_def.range.start;
    let end = field_def.range.end;
    let pad_with = field_def.pad_with;
    let justify = field_def.justify;

    quote! {
        fixed_width::FieldSet::new_field(#start..#end)
            .name(#name)
            .pad_with(#pad_with)
            .justify(#justify.to_string())
    }
}
