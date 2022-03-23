/// Helper macro for `FieldSet` createion with ease.
///
/// ### Example
///
/// ```rust
/// use fixed_width::{FieldSet, field, field_seq};
///
/// // Suppose we have fields definition as following:
/// let fields = FieldSet::Seq(
///     vec![
///         FieldSet::new_field(0..4).name("foo"),
///         FieldSet::Seq(
///             vec![
///                 FieldSet::new_field(4..6),
///                 FieldSet::new_field(6..8),
///             ]
///         ),
///     ]
/// );
///
/// // Which is identically with following:
/// let fields_with_macro = field_seq![
///     field!(0..4).name("foo"),
///     field_seq![
///         field!(4..6),
///         field!(6..8),
///     ]
/// ];
///
/// assert_eq!(format!("{:?}", fields), format!("{:?}", fields_with_macro));
/// ```
#[macro_export]
macro_rules! field_seq {
    ($($field:expr),+ $(,)?) => {
        FieldSet::Seq(vec![$($field),+])
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($range:expr) => {
        FieldSet::new_field($range)
    };
}
