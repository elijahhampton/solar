use crate::{
    ast_lowering::resolve::{Declaration, Declarations},
    hir,
    ty::{Gcx, Ty},
};
use solar_ast::ast::StateMutability as SM;
use solar_interface::{kw, sym, Span, Symbol};

pub(crate) mod members;
pub use members::{Member, MemberMap};

pub(crate) fn scopes() -> (Declarations, Box<[Option<Declarations>; Builtin::COUNT]>) {
    let global = declarations(Builtin::global().iter().copied());
    let inner = Box::new(std::array::from_fn(|i| {
        Some(declarations(Builtin::from_index(i).unwrap().inner()?.iter().copied()))
    }));
    (global, inner)
}

fn declarations(builtins: impl IntoIterator<Item = Builtin>) -> Declarations {
    let mut declarations = Declarations::new();
    for builtin in builtins {
        let decl = Declaration { kind: hir::Res::Builtin(builtin), span: Span::DUMMY };
        declarations.declarations.entry(builtin.name()).or_default().push(decl);
    }
    declarations
}

macro_rules! declare_builtins {
    (|$gcx:ident| $($(#[$variant_attr:meta])* $variant_name:ident => $sym:ident::$name:ident => $ty:expr;)*) => {
        /// A compiler builtin.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Builtin {
            $(
                $(#[$variant_attr])*
                $variant_name,
            )*
        }

        impl Builtin {
            pub const COUNT: usize = 0 $(+ { let _ = Builtin::$variant_name; 1 })*;

            /// Returns the symbol of the builtin.
            pub fn name(self) -> Symbol {
                match self {
                    $(
                        Builtin::$variant_name => $sym::$name,
                    )*
                }
            }

            /// Returns the type of the builtin.
            pub fn ty(self, $gcx: Gcx<'_>) -> Ty<'_> {
                match self {
                    $(
                        Builtin::$variant_name => $ty,
                    )*
                }
            }
        }
    };
}

declare_builtins! {
    |gcx|

    // Global
    Blockhash              => kw::Blockhash
                           => gcx.mk_builtin_fn(&[gcx.types.uint(256)], SM::View, &[gcx.types.fixed_bytes(32)]);
    Blobhash               => kw::Blobhash
                           => gcx.mk_builtin_fn(&[gcx.types.uint(256)], SM::View, &[gcx.types.fixed_bytes(32)]);

    Assert                 => sym::assert
                           => gcx.mk_builtin_fn(&[gcx.types.bool], SM::Pure, &[]);
    Require                => sym::require
                           => gcx.mk_builtin_fn(&[gcx.types.bool], SM::Pure, &[]);
    Revert                 => kw::Revert
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[]);
    RevertMsg              => kw::Revert
                           => gcx.mk_builtin_fn(&[gcx.types.string], SM::Pure, &[]);

    AddMod                 => kw::Addmod
                           => gcx.mk_builtin_fn(&[gcx.types.uint(256), gcx.types.uint(256), gcx.types.uint(256)], SM::Pure, &[gcx.types.uint(256)]);
    MulMod                 => kw::Mulmod
                           => gcx.mk_builtin_fn(&[gcx.types.uint(256), gcx.types.uint(256), gcx.types.uint(256)], SM::Pure, &[gcx.types.uint(256)]);

    Keccak256              => kw::Keccak256
                           => gcx.mk_builtin_fn(&[gcx.types.bytes_ref.memory], SM::View, &[gcx.types.fixed_bytes(32)]);
    Sha256                 => sym::sha256
                           => gcx.mk_builtin_fn(&[gcx.types.bytes_ref.memory], SM::View, &[gcx.types.fixed_bytes(32)]);
    Ripemd160              => sym::ripemd160
                           => gcx.mk_builtin_fn(&[gcx.types.bytes_ref.memory], SM::View, &[gcx.types.fixed_bytes(20)]);
    EcRecover              => sym::ecrecover
                           => gcx.mk_builtin_fn(&[gcx.types.fixed_bytes(32), gcx.types.uint(8), gcx.types.fixed_bytes(32), gcx.types.fixed_bytes(32)], SM::View, &[gcx.types.address]);

    Block                  => sym::block
                           => gcx.mk_builtin_mod(Self::Block);
    Msg                    => sym::msg
                           => gcx.mk_builtin_mod(Self::Msg);
    Tx                     => sym::tx
                           => gcx.mk_builtin_mod(Self::Tx);
    Abi                    => sym::abi
                           => gcx.mk_builtin_mod(Self::Abi);

    This                   => sym::this   => unreachable!();
    Super                  => sym::super_ => unreachable!();

    // `block`
    BlockCoinbase          => kw::Coinbase
                           => gcx.types.address_payable;
    BlockTimestamp         => kw::Timestamp
                           => gcx.types.uint(256);
    BlockDifficulty        => kw::Difficulty
                           => gcx.types.uint(256);
    BlockPrevrandao        => kw::Prevrandao
                           => gcx.types.uint(256);
    BlockNumber            => kw::Number
                           => gcx.types.uint(256);
    BlockGaslimit          => kw::Gaslimit
                           => gcx.types.uint(256);
    BlockChainid           => kw::Chainid
                           => gcx.types.uint(256);
    BlockBasefee           => kw::Basefee
                           => gcx.types.uint(256);
    BlockBlobbasefee       => kw::Blobbasefee
                           => gcx.types.uint(256);

    // `msg`
    MsgSender              => sym::sender
                           => gcx.types.address;
    MsgGas                 => kw::Gas
                           => gcx.types.uint(256);
    MsgValue               => sym::value
                           => gcx.types.uint(256);
    MsgData                => sym::data
                           => gcx.types.bytes_ref.calldata;
    MsgSig                 => sym::sig
                           => gcx.types.fixed_bytes(4);

    // `tx`
    TxOrigin               => kw::Origin
                           => gcx.types.address;
    TxGasPrice             => kw::Gasprice
                           => gcx.types.uint(256);

    // `abi`
    // TODO                => `(T...) pure returns(bytes memory)`
    AbiEncode              => sym::encode
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[gcx.types.bytes_ref.memory]);
    // TODO                => `(T...) pure returns(bytes memory)`
    AbiEncodePacked        => sym::encodePacked
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[gcx.types.bytes_ref.memory]);
    // TODO                => `(bytes4, T...) pure returns(bytes memory)`
    AbiEncodeWithSelector  => sym::encodeWithSelector
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[gcx.types.bytes_ref.memory]);
    // TODO                => `(F, T...) pure returns(bytes memory)`
    AbiEncodeCall          => sym::encodeCall
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[gcx.types.bytes_ref.memory]);
    // TODO                => `(string memory, T...) pure returns(bytes memory)`
    AbiEncodeWithSignature => sym::encodeWithSignature
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[gcx.types.bytes_ref.memory]);
    // TODO                => `(bytes memory, (T...)) pure returns(T...)`
    AbiDecode              => sym::decode
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[]);

    // --- impls ---

    AddressBalance         => kw::Balance
                           => gcx.types.uint(256);
    AddressCode            => sym::code
                           => gcx.types.bytes_ref.memory;
    AddressCodehash        => sym::codehash
                           => gcx.types.fixed_bytes(32);
    AddressCall            => kw::Call
                           => gcx.mk_builtin_fn(&[gcx.types.bytes_ref.memory], SM::View, &[gcx.types.bytes_ref.memory]);
    AddressDelegatecall    => kw::Delegatecall
                           => gcx.mk_builtin_fn(&[gcx.types.bytes_ref.memory], SM::View, &[gcx.types.bytes_ref.memory]);
    AddressStaticcall      => kw::Staticcall
                           => gcx.mk_builtin_fn(&[gcx.types.bytes_ref.memory], SM::View, &[gcx.types.bytes_ref.memory]);

    AddressPayableTransfer => sym::transfer
                           => gcx.mk_builtin_fn(&[gcx.types.uint(256)], SM::NonPayable, &[]);
    AddressPayableSend     => sym::send
                           => gcx.mk_builtin_fn(&[gcx.types.uint(256)], SM::NonPayable, &[gcx.types.bool]);

    FixedBytesLength       => sym::length
                           => gcx.types.uint(8);

    ArrayLength            => sym::length
                           => gcx.types.uint(256);

    ErrorSelector          => sym::selector
                           => gcx.types.fixed_bytes(4);

    EventSelector          => sym::selector
                           => gcx.types.fixed_bytes(32);

    // `type(T)`
    ContractCreationCode   => sym::creationCode
                           => gcx.types.bytes_ref.memory;
    ContractRuntimeCode    => sym::runtimeCode
                           => gcx.types.bytes_ref.memory;
    ContractName           => sym::name
                           => gcx.types.string_ref.memory;
    InterfaceId            => sym::interfaceId
                           => gcx.types.fixed_bytes(4);
    TypeMin                => sym::min => unreachable!();
    TypeMax                => sym::max => unreachable!();

    // `TyKind::Type` (`string.concat`, on the `string` type, not a string value)
    UdvtWrap               => sym::wrap   => unreachable!();
    UdvtUnwrap             => sym::unwrap => unreachable!();

    // TODO                => `(string memory...) pure returns(string memory)`
    StringConcat           => sym::concat
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[gcx.types.string_ref.memory]);

    // TODO                => `(bytes memory...) pure returns(bytes memory)`
    BytesConcat            => sym::concat
                           => gcx.mk_builtin_fn(&[], SM::Pure, &[gcx.types.bytes_ref.memory]);
}

impl Builtin {
    /// Returns an iterator over all builtins.
    #[inline]
    pub fn iter() -> std::iter::Map<std::ops::Range<usize>, impl FnMut(usize) -> Self> {
        (0..Self::COUNT).map(|i| Self::from_index(i).unwrap())
    }

    #[inline]
    fn from_index(i: usize) -> Option<Self> {
        if i < Self::COUNT {
            Some(unsafe { std::mem::transmute::<u8, Self>(i as u8) })
        } else {
            None
        }
    }

    /// Returns the global builtins.
    pub fn global() -> &'static [Self] {
        use Builtin::*;
        &[
            Blockhash, Blobhash, Assert, Require, Revert, RevertMsg, AddMod, MulMod, Keccak256,
            Sha256, Ripemd160, EcRecover, Block, Msg, Tx, Abi,
        ]
    }

    /// Returns the inner builtins.
    pub fn inner(self) -> Option<&'static [Self]> {
        use Builtin::*;
        Some(match self {
            Block => &[
                BlockCoinbase,
                BlockTimestamp,
                BlockDifficulty,
                BlockPrevrandao,
                BlockNumber,
                BlockGaslimit,
                BlockChainid,
                BlockBasefee,
                BlockBlobbasefee,
            ],
            Msg => &[MsgSender, MsgGas, MsgValue, MsgData, MsgSig],
            Tx => &[TxOrigin, TxGasPrice],
            Abi => &[
                AbiEncode,
                AbiEncodePacked,
                AbiEncodeWithSelector,
                AbiEncodeCall,
                AbiEncodeWithSignature,
                AbiDecode,
            ],
            _ => return None,
        })
    }
}
