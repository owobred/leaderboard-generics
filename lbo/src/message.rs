pub trait AuthoredMesasge {
    type Id;

    fn author_id(&self) -> Self::Id;
}
