pub trait Push<T> {
    fn push(&mut self, value: T);
    fn new_and_push(value: T) -> Self;
}

impl<T> Push<T> for Vec<T> {
    fn push(&mut self, value: T) {
        self.push(value);
    }
    fn new_and_push(value: T) -> Self {
        vec![value]
    }
}

impl Push<syn::Error> for syn::Error {
    fn push(&mut self, error: Self) {
        self.combine(error);
    }
    fn new_and_push(value: Self) -> Self {
        value
    }
}

pub trait TryCollectAll<T, CollectT, E, CollectE>: Iterator<Item = Result<T, E>> + Sized
where
    CollectT: Push<T>,
    CollectE: Push<E>,
{
    fn try_collect_all(mut self) -> Result<Option<CollectT>, CollectE> {
        let e = 'ok: {
            let mut collect_t = match self.next() {
                Some(Ok(t)) => CollectT::new_and_push(t),
                Some(Err(e)) => break 'ok e,
                None => return Ok(None),
            };
            for value in &mut self {
                match value {
                    Ok(t) => collect_t.push(t),
                    Err(e) => break 'ok e,
                }
            }
            return Ok(Some(collect_t));
        };
        let mut collect_e = CollectE::new_and_push(e);
        for value in self {
            if let Err(e) = value {
                collect_e.push(e);
            }
        }
        Err(collect_e)
    }
    fn try_collect_all_default(self) -> Result<CollectT, CollectE>
    where
        CollectT: Default,
    {
        self.try_collect_all().map(Option::unwrap_or_default)
    }
}

impl<I, T, CollectT, E, CollectE> TryCollectAll<T, CollectT, E, CollectE> for I
where
    I: Iterator<Item = Result<T, E>> + Sized,
    CollectT: Push<T> + Default,
    CollectE: Push<E>,
{
}
