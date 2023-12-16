use crate::{
    ser::{ScratchSpace, Serializer},
    string::{ArchivedString, StringResolver},
    Archive, Deserialize, Fallible, Serialize,
};
use smol_str::SmolStr;

impl Archive for SmolStr {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    #[inline]
    unsafe fn resolve(
        &self,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        ArchivedString::resolve_from_str(self, pos, resolver, out);
    }
}

impl<S> Serialize<S> for SmolStr
where
    S: Fallible + ScratchSpace + Serializer + ?Sized,
{
    #[inline]
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(self, serializer)
    }
}

impl<D: Fallible + ?Sized> Deserialize<SmolStr, D> for ArchivedString {
    #[inline]
    fn deserialize(&self, _deserializer: &mut D) -> Result<SmolStr, D::Error> {
        Ok(SmolStr::new(self.as_str()))
    }
}

impl PartialEq<SmolStr> for ArchivedString {
    fn eq(&self, other: &SmolStr) -> bool {
        other.as_str() == self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use crate::{access_unchecked, ser::Serializer, Deserialize};
    use rancor::{Infallible, Strategy, Failure};
    use smol_str::SmolStr;

    #[test]
    fn smolstr() {
        use crate::ser::serializers::CoreSerializer;

        let value = SmolStr::new("smol_str");

        let serializer = crate::util::serialize_into::<_, _, Failure>(
            &value,
            CoreSerializer::<256, 256>::default(),
        ).unwrap();
        let end = Serializer::<Failure>::pos(&serializer);
        let result = serializer.into_serializer().into_inner();
        let archived = unsafe { access_unchecked::<SmolStr>(&result[0..end]) };
        assert_eq!(archived, &value);

        let deserialized: SmolStr =
            archived.deserialize(Strategy::<_, Infallible>::wrap(&mut ())).unwrap();
        assert_eq!(value, deserialized);
    }
}
