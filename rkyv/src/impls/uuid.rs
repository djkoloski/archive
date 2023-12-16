use crate::{Archive, Deserialize, Fallible, Serialize};
use uuid::Uuid;

impl Archive for Uuid {
    type Archived = Uuid;
    type Resolver = ();

    unsafe fn resolve(
        &self,
        _: usize,
        _: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        // Safety: Uuid is portable and has no padding
        out.write(*self);
    }
}

// Safety: Uuid is portable and has no padding
#[cfg(feature = "copy")]
unsafe impl crate::copy::ArchiveCopySafe for Uuid {}

impl<S: Fallible + ?Sized> Serialize<S> for Uuid {
    fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible + ?Sized> Deserialize<Uuid, D> for Uuid {
    fn deserialize(&self, _: &mut D) -> Result<Uuid, D::Error> {
        Ok(*self)
    }
}

#[cfg(test)]
mod rkyv_tests {
    use crate::{
        access_unchecked,
        ser::serializers::AlignedSerializer,
        util::AlignedVec,
        Deserialize,
    };
    use rancor::{Strategy, Infallible};
    use uuid::Uuid;

    #[test]
    fn test_serialize_deserialize() {
        let uuid_str = "f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4";
        let u = Uuid::parse_str(uuid_str).unwrap();

        let serializer = crate::util::serialize_into::<_, _, Infallible>(
            &u,
            AlignedSerializer::new(AlignedVec::new()),
        ).expect("failed to archive uuid");
        let buf = serializer.into_inner();
        let archived = unsafe { access_unchecked::<Uuid>(buf.as_ref()) };

        assert_eq!(&u, archived);

        let deserialized = archived
            .deserialize(Strategy::<_, Infallible>::wrap(&mut ()))
            .expect("failed to deserialize uuid");

        assert_eq!(u, deserialized);
    }
}
