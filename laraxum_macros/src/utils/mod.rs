pub mod collections;
pub mod syn;

pub fn map_is_some<T, F>(value: &T, f: F) -> (&T, bool)
where
    F: FnOnce(&T) -> Option<&T>,
{
    let option = f(value);
    (option.unwrap_or(value), option.is_some())
}

// pub fn cow_try_and_then<'t, T, E, F>(cow: Cow<'t, T>, f: F) -> (Cow<'t, T>, Result<(), E>)
// where
//     T: ?Sized + ToOwned,
//     Cow<'t, T>: Borrow<T>,
//     F: for<'t1> FnOnce(&'t1 T) -> Result<Cow<'t1, T>, E>,
// {
//     match cow {
//         Cow::Owned(owned) => {
//             let cow2 = f(owned.borrow());
//             match cow2 {
//                 Ok(cow2) => (Cow::Owned(cow2.into_owned()), Ok(())),
//                 Err(err) => (Cow::Owned(owned), Err(err)),
//             }
//         }
//         Cow::Borrowed(borrowed) => {
//             let cow2 = f(borrowed);
//             match cow2 {
//                 Ok(cow2) => (cow2, Ok(())),
//                 Err(err) => (Cow::Borrowed(borrowed), Err(err)),
//             }
//         }
//     }
// }
//
// pub fn cow_try_and_then_option<'t, T, F>(cow: Cow<'t, T>, f: F) -> (Cow<'t, T>, Option<()>)
// where
//     T: ?Sized + ToOwned,
//     Cow<'t, T>: Borrow<T>,
//     F: for<'t1> FnOnce(&'t1 T) -> Option<Cow<'t1, T>>,
// {
//     let (cow, result) = cow_try_and_then(cow, |cow| f(cow).ok_or(()));
//     (cow, result.ok())
// }
// pub fn cow_try_and_then_is_some<'t, T, F>(cow: Cow<'t, T>, f: F) -> (Cow<'t, T>, bool)
// where
//     T: ?Sized + ToOwned,
//     Cow<'t, T>: Borrow<T>,
//     F: for<'t1> FnOnce(&'t1 T) -> Option<Cow<'t1, T>>,
// {
//     let (cow, result) = cow_try_and_then(cow, |cow| f(cow).ok_or(()));
//     (cow, result.is_ok())
// }
