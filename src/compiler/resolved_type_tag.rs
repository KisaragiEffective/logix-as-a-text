enum TypeTag {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    String,
    Impulse,
    Bool,
    BoundingBox,
    Char16,
    Char32,
    Color,
    DateTime,
    Decimal,
    DoubleQuotanion,
    FloatQuotanion,
    Slot,
    User,
    IValue {
        inner: Box<Self>,
    },
    IField {
        inner: Box<Self>,
    },
    SyncPlayback,
    IWorldElement,
    SyncRef {
        inner: Box<Self>,
    },
    Uri,
    StaticAudioClipProvider,
    StaticMesh,
    SpriteProvider,
    StaticTexture2D,
    IAssetProvider {
        inner: Box<Self>,
    },
    AvatarAnchor,
    IFingerPoseSource,
    IComponent,
    /// corresponds with "dummy" type.
    ToBeInferred,
    Matrix1D {
        element_count: MatrixElementCount,
        type_tag: Matrix1DTypeTag,
    },
    Matrix2D {
        element_count: MatrixElementCount,
        type_tag: Matrix2DTypeTag,
    }
}

enum MatrixElementCount {
    Two,
    Three,
    Four,
}

enum Matrix1DTypeTag {
    Bool,
    F64,
    F32,
    I32,
    I64,
    U32,
    U64,
}

enum Matrix2DTypeTag {
    F64,
    F32,
}
