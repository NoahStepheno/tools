use super::{Buffer, Format, Formatter};
use crate::FormatResult;
use std::ffi::c_void;
use std::marker::PhantomData;

/// Mono-morphed type to format an object. Used by the [crate::format!], [crate::format_args!], and
/// [crate::write!] macros.
///
/// This struct is similar to a dynamic dispatch (using `dyn Format`) because it stores a pointer to the value.
/// However, it doesn't store the pointer to `dyn Format`'s vtable, instead it statically resolves the function
/// pointer of `Format::format` and stores it in `formatter`.
pub struct Argument<'fmt, Context> {
    /// The value to format stored as a raw pointer where `lifetime` stores the value's lifetime.
    value: *const c_void,

    /// Stores the lifetime of the value. To get the most out of our dear borrow checker.
    lifetime: PhantomData<&'fmt ()>,

    /// The function pointer to `value`'s `Format::format` method
    formatter: fn(*const c_void, &mut Formatter<'_, Context>) -> FormatResult<()>,
}

impl<Context> Clone for Argument<'_, Context> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Context> Copy for Argument<'_, Context> {}

impl<'fmt, Context> Argument<'fmt, Context> {
    /// Called by the [rome_formatter::format_args] macro. Creates a mono-morphed value for formatting
    /// an object.
    #[doc(hidden)]
    #[inline]
    pub fn new<F: Format<Context>>(value: &'fmt F) -> Self {
        fn formatter<F: Format<Context>, Context>(
            ptr: *const c_void,
            fmt: &mut Formatter<Context>,
        ) -> FormatResult<()> {
            // SAFETY: Safe because the 'fmt lifetime is captured by the 'lifetime' field.
            F::fmt(unsafe { &*(ptr as *const F) }, fmt)
        }

        Self {
            value: value as *const F as *const c_void,
            lifetime: PhantomData,
            formatter: formatter::<F, Context>,
        }
    }

    /// Formats the value stored by this argument using the given formatter.
    #[inline]
    pub(super) fn format(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        (self.formatter)(self.value, f)
    }
}

/// Sequence of objects that should be formatted in the specified order.
///
/// The [`format_args!`] macro will safely create an instance of this structure.
///
/// You can use the `Arguments<a>` that [`format_args!]` return in `Format` context as seen below.
/// It will call the `format` function for every of it's objects.
///
/// ```rust
/// use rome_formatter::prelude::*;
/// use rome_formatter::{format, format_args};
///
/// let formatted = format!(SimpleFormatContext::default(), [
///     format_args!(text("a"), space(), text("b"))
/// ]).unwrap();
///
/// assert_eq!("a b", formatted.print().as_code());
/// ```
pub struct Arguments<'fmt, Context>(pub &'fmt [Argument<'fmt, Context>]);

impl<'fmt, Context> Arguments<'fmt, Context> {
    #[doc(hidden)]
    #[inline]
    pub fn new(arguments: &'fmt [Argument<'fmt, Context>]) -> Self {
        Self(arguments)
    }

    /// Returns the arguments
    #[inline]
    pub(super) fn items(&self) -> &'fmt [Argument<'fmt, Context>] {
        self.0
    }
}

impl<Context> Copy for Arguments<'_, Context> {}

impl<Context> Clone for Arguments<'_, Context> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<Context> Format<Context> for Arguments<'_, Context> {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<Context>) -> FormatResult<()> {
        formatter.write_fmt(*self)
    }
}

impl<Context> std::fmt::Debug for Arguments<'_, Context> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Arguments[...]")
    }
}

impl<'fmt, Context> From<&'fmt Argument<'fmt, Context>> for Arguments<'fmt, Context> {
    fn from(argument: &'fmt Argument<'fmt, Context>) -> Self {
        Arguments::new(std::slice::from_ref(argument))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::{format_args, format_element, write, FormatState, VecBuffer};

    #[test]
    fn test_nesting() {
        let mut context = FormatState::new(());
        let mut buffer = VecBuffer::new(&mut context);

        write!(
            &mut buffer,
            [
                text("function"),
                space(),
                text("a"),
                space(),
                group(&format_args!(text("("), text(")")))
            ]
        )
        .unwrap();

        assert_eq!(
            buffer.into_element(),
            FormatElement::List(List::new(vec![
                FormatElement::Text(Text::Static { text: "function" }),
                FormatElement::Space,
                FormatElement::Text(Text::Static { text: "a" }),
                FormatElement::Space,
                FormatElement::Group(format_element::Group::new(vec![
                    FormatElement::Text(Text::Static { text: "(" }),
                    FormatElement::Text(Text::Static { text: ")" }),
                ]))
            ]))
        );
    }
}
