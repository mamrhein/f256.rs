// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{Float256, FP492};
use crate::f256;

const N: usize = 33;

const COEFFS: [FP492; N] = [
    // 65: 1 / 65
    FP492::new(
        0x0000003f03f03f03f03f03f03f03f03f,
        0x03f03f03f03f03f03f03f03f03f03f03,
        0xf03f03f03f03f03f03f03f03f03f03f0,
        0x3f03f03f03f03f03f03f03f03f03f03f,
    ),
    // 63: -1 / 63
    FP492::new(
        0xffffffbefbefbefbefbefbefbefbefbe,
        0xfbefbefbefbefbefbefbefbefbefbefb,
        0xefbefbefbefbefbefbefbefbefbefbef,
        0xbefbefbefbefbefbefbefbefbefbefbf,
    ),
    // 61: 1 / 61
    FP492::new(
        0x0000004325c53ef368eb04325c53ef36,
        0x8eb04325c53ef368eb04325c53ef368e,
        0xb04325c53ef368eb04325c53ef368eb0,
        0x4325c53ef368eb04325c53ef368eb043,
    ),
    // 59: -1 / 59
    FP492::new(
        0xffffffba9386822b63cbeea4e1a08ad8,
        0xf2fba9386822b63cbeea4e1a08ad8f2f,
        0xba9386822b63cbeea4e1a08ad8f2fba9,
        0x386822b63cbeea4e1a08ad8f2fba9387,
    ),
    // 57: 1 / 57
    FP492::new(
        0x00000047dc11f7047dc11f7047dc11f7,
        0x047dc11f7047dc11f7047dc11f7047dc,
        0x11f7047dc11f7047dc11f7047dc11f70,
        0x47dc11f7047dc11f7047dc11f7047dc1,
    ),
    // 55: -1 / 55
    FP492::new(
        0xffffffb586fb586fb586fb586fb586fb,
        0x586fb586fb586fb586fb586fb586fb58,
        0x6fb586fb586fb586fb586fb586fb586f,
        0xb586fb586fb586fb586fb586fb586fb6,
    ),
    // 53: 1 / 53
    FP492::new(
        0x0000004d4873ecade304d4873ecade30,
        0x4d4873ecade304d4873ecade304d4873,
        0xecade304d4873ecade304d4873ecade3,
        0x04d4873ecade304d4873ecade304d487,
    ),
    // 51: -1 / 51
    FP492::new(
        0xffffffafafafafafafafafafafafafaf,
        0xafafafafafafafafafafafafafafafaf,
        0xafafafafafafafafafafafafafafafaf,
        0xafafafafafafafafafafafafafafafb0,
    ),
    // 49: 1 / 49
    FP492::new(
        0x0000005397829cbc14e5e0a72f053978,
        0x29cbc14e5e0a72f05397829cbc14e5e0,
        0xa72f05397829cbc14e5e0a72f0539782,
        0x9cbc14e5e0a72f05397829cbc14e5e0a,
    ),
    // 47: -1 / 47
    FP492::new(
        0xffffffa8d9df51b3bea3677d46cefa8d,
        0x9df51b3bea3677d46cefa8d9df51b3be,
        0xa3677d46cefa8d9df51b3bea3677d46c,
        0xefa8d9df51b3bea3677d46cefa8d9df5,
    ),
    // 45: 1 / 45
    FP492::new(
        0x0000005b05b05b05b05b05b05b05b05b,
        0x05b05b05b05b05b05b05b05b05b05b05,
        0xb05b05b05b05b05b05b05b05b05b05b0,
        0x5b05b05b05b05b05b05b05b05b05b05b,
    ),
    // 43: -1 / 43
    FP492::new(
        0xffffffa0be82fa0be82fa0be82fa0be8,
        0x2fa0be82fa0be82fa0be82fa0be82fa0,
        0xbe82fa0be82fa0be82fa0be82fa0be82,
        0xfa0be82fa0be82fa0be82fa0be82fa0c,
    ),
    // 41: 1 / 41
    FP492::new(
        0x00000063e7063e7063e7063e7063e706,
        0x3e7063e7063e7063e7063e7063e7063e,
        0x7063e7063e7063e7063e7063e7063e70,
        0x63e7063e7063e7063e7063e7063e7064,
    ),
    // 39: -1 / 39
    FP492::new(
        0xffffff96f96f96f96f96f96f96f96f96,
        0xf96f96f96f96f96f96f96f96f96f96f9,
        0x6f96f96f96f96f96f96f96f96f96f96f,
        0x96f96f96f96f96f96f96f96f96f96f97,
    ),
    // 37: 1 / 37
    FP492::new(
        0x0000006eb3e45306eb3e45306eb3e453,
        0x06eb3e45306eb3e45306eb3e45306eb3,
        0xe45306eb3e45306eb3e45306eb3e4530,
        0x6eb3e45306eb3e45306eb3e45306eb3e,
    ),
    // 35: -1 / 35
    FP492::new(
        0xffffff8af8af8af8af8af8af8af8af8a,
        0xf8af8af8af8af8af8af8af8af8af8af8,
        0xaf8af8af8af8af8af8af8af8af8af8af,
        0x8af8af8af8af8af8af8af8af8af8af8b,
    ),
    // 33: 1 / 33
    FP492::new(
        0x0000007c1f07c1f07c1f07c1f07c1f07,
        0xc1f07c1f07c1f07c1f07c1f07c1f07c1,
        0xf07c1f07c1f07c1f07c1f07c1f07c1f0,
        0x7c1f07c1f07c1f07c1f07c1f07c1f07c,
    ),
    // 31: -1 / 31
    FP492::new(
        0xffffff7bdef7bdef7bdef7bdef7bdef7,
        0xbdef7bdef7bdef7bdef7bdef7bdef7bd,
        0xef7bdef7bdef7bdef7bdef7bdef7bdef,
        0x7bdef7bdef7bdef7bdef7bdef7bdef7c,
    ),
    // 29: 1 / 29
    FP492::new(
        0x0000008d3dcb08d3dcb08d3dcb08d3dc,
        0xb08d3dcb08d3dcb08d3dcb08d3dcb08d,
        0x3dcb08d3dcb08d3dcb08d3dcb08d3dcb,
        0x08d3dcb08d3dcb08d3dcb08d3dcb08d4,
    ),
    // 27: -1 / 27
    FP492::new(
        0xffffff684bda12f684bda12f684bda12,
        0xf684bda12f684bda12f684bda12f684b,
        0xda12f684bda12f684bda12f684bda12f,
        0x684bda12f684bda12f684bda12f684be,
    ),
    // 25: 1 / 25
    FP492::new(
        0x000000a3d70a3d70a3d70a3d70a3d70a,
        0x3d70a3d70a3d70a3d70a3d70a3d70a3d,
        0x70a3d70a3d70a3d70a3d70a3d70a3d70,
        0xa3d70a3d70a3d70a3d70a3d70a3d70a4,
    ),
    // 23: -1 / 23
    FP492::new(
        0xffffff4de9bd37a6f4de9bd37a6f4de9,
        0xbd37a6f4de9bd37a6f4de9bd37a6f4de,
        0x9bd37a6f4de9bd37a6f4de9bd37a6f4d,
        0xe9bd37a6f4de9bd37a6f4de9bd37a6f5,
    ),
    // 21: 1 / 21
    FP492::new(
        0x000000c30c30c30c30c30c30c30c30c3,
        0x0c30c30c30c30c30c30c30c30c30c30c,
        0x30c30c30c30c30c30c30c30c30c30c30,
        0xc30c30c30c30c30c30c30c30c30c30c3,
    ),
    // 19: -1 / 19
    FP492::new(
        0xffffff286bca1af286bca1af286bca1a,
        0xf286bca1af286bca1af286bca1af286b,
        0xca1af286bca1af286bca1af286bca1af,
        0x286bca1af286bca1af286bca1af286bd,
    ),
    // 17: 1 / 17
    FP492::new(
        0x000000f0f0f0f0f0f0f0f0f0f0f0f0f0,
        0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0,
        0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0,
        0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f1,
    ),
    // 15: -1 / 15
    FP492::new(
        0xfffffeeeeeeeeeeeeeeeeeeeeeeeeeee,
        0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee,
        0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee,
        0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeef,
    ),
    // 13: 1 / 13
    FP492::new(
        0x0000013b13b13b13b13b13b13b13b13b,
        0x13b13b13b13b13b13b13b13b13b13b13,
        0xb13b13b13b13b13b13b13b13b13b13b1,
        0x3b13b13b13b13b13b13b13b13b13b13b,
    ),
    // 11: -1 / 11
    FP492::new(
        0xfffffe8ba2e8ba2e8ba2e8ba2e8ba2e8,
        0xba2e8ba2e8ba2e8ba2e8ba2e8ba2e8ba,
        0x2e8ba2e8ba2e8ba2e8ba2e8ba2e8ba2e,
        0x8ba2e8ba2e8ba2e8ba2e8ba2e8ba2e8c,
    ),
    // 9: 1 / 9
    FP492::new(
        0x000001c71c71c71c71c71c71c71c71c7,
        0x1c71c71c71c71c71c71c71c71c71c71c,
        0x71c71c71c71c71c71c71c71c71c71c71,
        0xc71c71c71c71c71c71c71c71c71c71c7,
    ),
    // 7: -1 / 7
    FP492::new(
        0xfffffdb6db6db6db6db6db6db6db6db6,
        0xdb6db6db6db6db6db6db6db6db6db6db,
        0x6db6db6db6db6db6db6db6db6db6db6d,
        0xb6db6db6db6db6db6db6db6db6db6db7,
    ),
    // 5: 1 / 5
    FP492::new(
        0x00000333333333333333333333333333,
        0x33333333333333333333333333333333,
        0x33333333333333333333333333333333,
        0x33333333333333333333333333333333,
    ),
    // 3: -1 / 3
    FP492::new(
        0xfffffaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab,
    ),
    // 1: 1
    FP492::new(
        0x00001000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
];

const ATANS: [FP492; 256] = [
    // atan(0/256)
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
    // atan(1/256)
    FP492::new(
        0x0000000ffffaaaaddddb94bb12afb6b6,
        0xd4f7e167c18baeb9bc957892248ad268,
        0x2bef66d688680f625cdfbd62450d6612,
        0x6f5a9d89ebbe467be3ae1471a535c801,
    ),
    // atan(2/256)
    FP492::new(
        0x0000001fffd555bbba972d00c46a3f77,
        0xcc15e8ed0ad402e345e003507a2aecfd,
        0xd9ca12111d08128bc8f596f9cb7c5ac4,
        0xa4e52cc7790ed9717e1e5570f29f8d7a,
    ),
    // atan(3/256)
    FP492::new(
        0x0000002fff700309861346d7f457bee5,
        0xa5b716f3b42e41beddd8e14db0e87ac4,
        0xd1faddc4ab848ebea3e479e608b5c912,
        0x94bed46af6bbfe41863021b37431d61f,
    ),
    // atan(4/256)
    FP492::new(
        0x0000003ffeaab776e5356ef9e3159005,
        0x7dd812083bd970437bbd181a57e00c9d,
        0x5872ea36a1de9858d1e5768d09936286,
        0xe9ba4bcabe016797ad49485a1b8fb4e0,
    ),
    // atan(5/256)
    FP492::new(
        0x0000004ffd657c629bfedec153c9304d,
        0x3fd27fd306b025200eb1056335a58a31,
        0x3278438d13644c5de3191734d22a1165,
        0xa6a5a3293c311206a07856b52a899a5a,
    ),
    // atan(6/256)
    FP492::new(
        0x0000005ffb80612970d6bce7603f5cab,
        0x5251b8dd4fb816aaedaa2c49cd2154ac,
        0x309136e211e5389a224cf36a7ad69c0b,
        0xef870a9c8faabcba0219c83a73639a09,
    ),
    // atan(7/256)
    FP492::new(
        0x0000006ff8db7ca45c470f209c454f7f,
        0x8cafccb479a5c6171a4ba9aaafeff02f,
        0xfdb3c63253408549887dac64cbc9c175,
        0xe84fc8750f3a2a501ca08d7aa9735c47,
    ),
    // atan(8/256)
    FP492::new(
        0x0000007ff556eea5d892a13bcebbb6ed,
        0x463109c036814a606dc40b2380beda26,
        0xb08303b2a91f5897e5f9d0f282ff5f7a,
        0x43e2d13c86927c276afc41358c95c6f3,
    ),
    // atan(9/256)
    FP492::new(
        0x0000008ff0d2e17624bb10c123d1f899,
        0x68a2aa4ff745e1bc682b8381d3599379,
        0xee4bed1bc015675f9682474fc920da9d,
        0xcd26e237c9daf29974e6bfc7e1c8c87a,
    ),
    // atan(10/256)
    FP492::new(
        0x0000009feb2f8b4e4ec8b918538b230d,
        0xa72f6184ecc9b220db88c0729939d7fd,
        0x2da020a0e82e97d6fac1932971e8ba2c,
        0x015f4aef449a6eee6b8d2993c1e8659d,
    ),
    // atan(11/256)
    FP492::new(
        0x000000afe44d2fd1d96a576e77e9cff3,
        0x171483c64d55b525a72aab6bd82767c2,
        0x8c6a29b0205875f2e123cec4b00d9fca,
        0xf53f60adf3417c991b20589362e3fb03,
    ),
    // atan(12/256)
    FP492::new(
        0x000000bfdc0c2186d14fcf220e10d61d,
        0xf56ec71dddd64f807f208a773120217d,
        0xe7976a5989626439bbf1eed0894b8671,
        0x7017d9b14ed68d755b36e257b0e0e163,
    ),
    // atan(13/256)
    FP492::new(
        0x000000cfd24cc34c26fa64520f5e1578,
        0x395ba5847020d6fddb3edb29c65b4e6d,
        0x80f3a8fe006591e707ffe02748cfd1f0,
        0xd18782d70dacadfbb2b41068a96f1e37,
    ),
    // atan(14/256)
    FP492::new(
        0x000000dfc6ef89ce221cec03969ab00b,
        0x0ebac9a51f29e95a4be500d1b3e5ca89,
        0x94635b5ffc5d057af8bc6be000539d44,
        0xf2ce72ef90dd074a3fd6eef5ac7c0861,
    ),
    // atan(15/256)
    FP492::new(
        0x000000efb9d4fcf8c40fb6b928efbd0d,
        0x2fcff6f59d19ec0fda8e7a582d3aea07,
        0xbc5f600c865034a1a6eb684887d3c544,
        0xce10ad19de2a998dcfd4f3c9d4d00d17,
    ),
    // atan(16/256)
    FP492::new(
        0x000000ffaaddb967ef4e36cb2792dc0e,
        0x2e0d51319c12cf59d4b2dc387a9f803c,
        0x4b8aed0249009473e9b7d493fe2926ac,
        0x74803b1404b3e625a4c0a42521de94ef,
    ),
    // atan(17/256)
    FP492::new(
        0x0000010f99ea71d52a6f6fcb0089e4ed,
        0x8919d8cfb4ced48fa3ec06901c519477,
        0xb4061c6b9bf91665127c474fdbeb38f1,
        0xc77375db92215563610b904dfff70311,
    ),
    // atan(18/256)
    FP492::new(
        0x0000011f86dbf082d58ded447119a20f,
        0xdb0fe529d558c1e15a474e438050ace6,
        0x50b85f3fbc7687fdd85df947dfc5204d,
        0x7c086655ccfe7eff0326b050b6c319ce,
    ),
    // atan(19/256)
    FP492::new(
        0x0000012f719318a4a9a027fa2ef37300,
        0x3e5caa22519ec518b299a00e6fba3c07,
        0x12f54e392fa225cf7d5a0e60a1616b08,
        0x2f40af832c092f279888242f420b7aa0,
    ),
    // atan(20/256)
    FP492::new(
        0x0000013f59f0e7c559d6b1338a177e11,
        0xcd9bec9eb30fb4bf3790cdc6b617d498,
        0x18882078cb0906d3c33b436af16c3ee1,
        0x7ef028ed6a6f3c97d6dbfa5a33de56f6,
    ),
    // atan(21/256)
    FP492::new(
        0x0000014f3fd677292fb2011a6c4c92f3,
        0xb2f1be4d45ea9ea17484d3eef315fe1e,
        0x69218ab6386654f11ad49b76dfb4a663,
        0xe28290b1311aeee2f955731bfc67969d,
    ),
    // atan(22/256)
    FP492::new(
        0x0000015f2324fd2d7b262a3691004c63,
        0x7eace735fa0c51c0afac18510be0ead1,
        0x08b54360e10c3e97341e8c0a38987f17,
        0xa2f213bbfe2d2d997e93c3756c2f945a,
    ),
    // atan(23/256)
    FP492::new(
        0x0000016f03bdcea4b0cec0ff1aed05e8,
        0x35fd94922f357b1dd745cb14cd4e02d0,
        0x8e8b8f6feaa8e2ea402ba262264e2e13,
        0xab974bbf07cdd0abf39789314ca64fe4,
    ),
    // atan(24/256)
    FP492::new(
        0x0000017ee182602f10e8c126acfcf099,
        0xf06cecfc1508f3055c1b865667e058c3,
        0x3e00b45474c82a4933246c7006c6bcfc,
        0x4eed0b05362dd147ccad0f61cb520e9f,
    ),
    // atan(25/256)
    FP492::new(
        0x0000018ebc54478fb282e6510195a499,
        0x750dec78b762383909b1ebd86f3b623c,
        0x4c8d01b94fe5bd5dfcebb38d9db2e6f8,
        0x69e7a2566ac69d47cc58234f12b4b4a9,
    ),
    // atan(26/256)
    FP492::new(
        0x0000019e94153cfdcf168ccb875a711f,
        0x8151e9fa90ca2727fec4a8c91e6e324f,
        0x4e839c1d8bf6a7eb2f5a907755f5edc3,
        0xae08dbb9fe797db955cd92b5d2c7776b,
    ),
    // atan(27/256)
    FP492::new(
        0x000001ae68a71c722b838029cd22173b,
        0x6e5bfbbfe48a9d88eba112c70ba3e798,
        0x1aea1ee88271500b991bca9a50ad8597,
        0xb7d78fa19f142d2cb14846d2b4b246c0,
    ),
    // atan(28/256)
    FP492::new(
        0x000001be39ebe6f07c37dee3ca681661,
        0xcbb3dd21afca1d234427db1b0e0bd686,
        0x64f7491b36f11e10e85eebf2c072e9a9,
        0x7042bce55540a0f7d48067b563833099,
    ),
    // atan(29/256)
    FP492::new(
        0x000001ce07c5c3cca322271cc84b2314,
        0xd28c1967b9dd827c7e505b6d450fa5cb,
        0x44d3e14cb3da88d2ed6acfc546c0e770,
        0x7b8254c94c1a1c0aca8c59e3f5b6d42c,
    ),
    // atan(30/256)
    FP492::new(
        0x000001ddd21701eba6e653bff35dbf95,
        0xfab59910b169af3435b28b68ef4979bc,
        0x0c871028bfa2c67d74e9e46e04ecc4d6,
        0x5e896ad29bec88d550940859d7037358,
    ),
    // atan(31/256)
    FP492::new(
        0x000001ed98c2190043af62d33d369c27,
        0x6a948e9e36d8465679841fd75ea0e76c,
        0x0b90a6fb3e1cab8999195830963f5bb6,
        0x750b52e37a5feca2d57097953bff97e2,
    ),
    // atan(32/256)
    FP492::new(
        0x000001fd5ba9aac2f6dc65912f313e7d,
        0x111def1672afb2bb35b245d926aefbf6,
        0xd82ed1a9a0100403b384ca7ff4baa3c4,
        0x69ae3abae79313e535f7fa54d9b1157f,
    ),
    // atan(33/256)
    FP492::new(
        0x0000020d1ab0842567b30e96be7f3201,
        0x76b74fcb8fbed65afedb4b1bc39a3fac,
        0x4ccb1fce00e462c350133793e76e9bee,
        0x963be7df6d8e914efc8755547ed246a8,
    ),
    // atan(34/256)
    FP492::new(
        0x0000021cd5b99e811031472e72233979,
        0x639684d87af78f809142886f8758604a,
        0x83365ea493561e62ac41e217f8cbaa57,
        0x870c3db4372f929474992812176288e7,
    ),
    // atan(35/256)
    FP492::new(
        0x0000022c8ca820c10a0b5e774573f548,
        0xd5e3062929db777f89387f93b0c5bc68,
        0xfa69be22637537dbf048484d054d029e,
        0x9f297b2df3ca8b4114288362f005263c,
    ),
    // atan(36/256)
    FP492::new(
        0x0000023c3f5f6086e4dc96f4dd64a60e,
        0x82be678a856b0a7f003239802c41c0be,
        0x90468d86815e7981f40daf6876258ccf,
        0x5586e7c24f7db6a7027102c25d670d26,
    ),
    // atan(37/256)
    FP492::new(
        0x0000024bedc2e34a6b8be36bdef0d188,
        0x3ce1cad71a122103b7390029970c1999,
        0xe00895df2d3e0b5839fc73dc8067c41d,
        0xe86b10cff80117d81bd767ada35c522a,
    ),
    // atan(38/256)
    FP492::new(
        0x0000025b97b65f743fe63ca1c155854d,
        0x14fcab40fd271ac3c6138e21eff2c091,
        0xff2129135bb99093a0e35f23f2d2f412,
        0xef49882d4695f78ebc7f75352cf1c80f,
    ),
    // atan(39/256)
    FP492::new(
        0x0000026b3d1dbd733472d01458dcfd9c,
        0xe2bd30b5bd245b577ec203c856851fdc,
        0x4387dcd667d187ec429b6e13ad104b30,
        0x7df04c616945c1c096d9ac31a5854944,
    ),
    // atan(40/256)
    FP492::new(
        0x0000027adddd18cc4d8b0d1d8674940d,
        0x83fa15dd4bd3e2eb74a37132f0292d0a,
        0x3c05b6d60ea3bbc6aa4fcde49b5b0a8a,
        0xdca3fbb480a41c6fdac72a4d22e76be3,
    ),
    // atan(41/256)
    FP492::new(
        0x0000028a79d8c12553c8fb29d9d2d3c5,
        0xb858fd95008299165ca72790d4bc43bd,
        0x019c4133b2b30d56d6f43f242275b65e,
        0xc851b62f2a8a9653615f42fe9defee47,
    ),
    // atan(42/256)
    FP492::new(
        0x0000029a10f53b49e2e8f991f8d6f8ab,
        0xcc1f785f820a0a602974f71e7b47d3fc,
        0xb54d0168365a5201dcfa4ce0f5b5d7c2,
        0x7bdc8630e9967d2c966a8a494772ab25,
    ),
    // atan(43/256)
    FP492::new(
        0x000002a9a317422ae148c14179442039,
        0xbaca952f68c3d18607febf998a3efc4f,
        0x7960778b884a9feffa81560b49f616e8,
        0x306a92bb9592d9871c02b51f79d6672a,
    ),
    // atan(44/256)
    FP492::new(
        0x000002b93023c7d84d3bead534ffbc30,
        0xb7a650b4f9b7546c1ad3307d8d94cf49,
        0xef5302514c715eb67c8dca4fd6da4c39,
        0xcd9f1bf80dddb0101678b0d4661db702,
    ),
    // atan(45/256)
    FP492::new(
        0x000002c8b7fff6754e7e0ac949883d34,
        0xc6f1118fddf54c6d28096444ea8d2b76,
        0x6dc2cdb25929a3b04b204bb0f3ba3118,
        0xac689953f1ebcd62dba77eb2e71d3748,
    ),
    // atan(46/256)
    FP492::new(
        0x000002d83a9131267b1b5aad627f0ef6,
        0x549a0b4ffb8c879083c3e1c29127f218,
        0xab4d978ef838178eca1e3ba26831a1e9,
        0x85042b94ffed467d27ea11ff62734bb8,
    ),
    // atan(47/256)
    FP492::new(
        0x000002e7b7bd14fa403a94bc24ad6668,
        0xd5f078b62242977f6c0a1a4c21996009,
        0x2d34d2cab1d2417eb0d22fff7147c644,
        0x347071b5a9ab0700b6577da82afeb0a5,
    ),
    // atan(48/256)
    FP492::new(
        0x000002f72f6979cb6044d1ec2d3e2072,
        0x71d21e4eb4035a0e28acabc169a93c79,
        0xf5cb465c41183a1314740a2e0e37d3f5,
        0xc02f3e354604108d5df802a81b45b550,
    ),
    // atan(49/256)
    FP492::new(
        0x00000306a17c731d78fa95bb33518dbc,
        0xc20fb83aa95920ade66d4edc5992388b,
        0xa6f5ff9217c225eb3d2df8a3881658ac,
        0xc7f9648f26c41bf0955fb2bffbcceff0,
    ),
    // atan(50/256)
    FP492::new(
        0x000003160ddc50f38517739967e7b9d8,
        0x5f2c5a797d90c339400bdeeb9d3f7a36,
        0xd0e0c9ba2a914f8da54e8537477a50a7,
        0xa037a0645863e1d8a38a4d021c10d43c,
    ),
    // atan(51/256)
    FP492::new(
        0x00000325746fa0a04e37961097805c4e,
        0x1ec71bfd3e8e910efb2cb784176a4d62,
        0x1348edf66b6f541ef746b37122f0d734,
        0xadac3222f6b01d8dce9c5e5f60493c8a,
    ),
    // atan(52/256)
    FP492::new(
        0x00000334d51d2d90c4c39ec03cf68691,
        0xbbaceaafc499306f09d86fc1796da222,
        0x4cf0a3e50601a64c501b484f6edfe1b8,
        0x356baa58182f3b1cd2237bf646ff85dd,
    ),
    // atan(53/256)
    FP492::new(
        0x000003442fcc021034b79083acfa8646,
        0x3cf3b8d8a7ad78664d9d84e0f9b08929,
        0x270714d93ee4b993cb59188d7a6c934b,
        0xd93bd1ec022ddcfd71a5b466afaf5908,
    ),
    // atan(54/256)
    FP492::new(
        0x00000353846368064f28745df44e6542,
        0xa87586909c14513c78e6582ec8d0975b,
        0xd3ea29f24f2e8b140d9ee24acd3294af,
        0x9b37aee7436586ee3bc74482c2e08b89,
    ),
    // atan(55/256)
    FP492::new(
        0x00000362d2cae9af0179e951c0fa9663,
        0x3f48ac9f4e6ac180edaf83b878193a1b,
        0x814a0992be0f07d8052da91d9eaa5da1,
        0xd01024e1e1f8cda7b7f4da9e22f003cb,
    ),
    // atan(56/256)
    FP492::new(
        0x000003721aea524c14408bd88697072d,
        0x54bc0a19144a34e92c49555881bc65fe,
        0x10a3b10c6abd0137701d52a7d62b5bba,
        0xce5d81bf788c79be36135d5f2f22e2d9,
    ),
    // atan(57/256)
    FP492::new(
        0x000003815ca9aed08cd7d99205506c92,
        0x13bed92f30bc530228d390be927c77ff,
        0x03accf78257e74023a354f2b6db3c08b,
        0xf886cf32ad755a2a3c8ee71bcfb8f0b6,
    ),
    // atan(58/256)
    FP492::new(
        0x0000039097f14e85cdb9a908be5f4576,
        0x6d521969f79c2c23af573630280dd546,
        0xb7e08077a90b894f88a2935845ab6efb,
        0xed4bf05c7d4324b3757b888ba384834a,
    ),
    // atan(59/256)
    FP492::new(
        0x0000039fcca9c3aa72aa3dcd1fb8f57d,
        0xac766cc23f3b11f847c4c5c89eeb2345,
        0x9a44be497e262fb49c05960822f91e0d,
        0xdcd03097865dd63714ab4ad8c32cf091,
    ),
    // atan(60/256)
    FP492::new(
        0x000003aefabbe40ae6ce32468a9a2cbe,
        0xf5e39ec4b3b0a80cbfc0179b6867e8a2,
        0x76c061e4e4bd1560f6205b8c3e75fd7c,
        0xa6cbcc4a7522b843bc77ef96efef59ca,
    ),
    // atan(61/256)
    FP492::new(
        0x000003be2210c993b3bea7e74ac40b87,
        0xf65091c68abe98b7a73c31d0cbb9f711,
        0x0586aec406449bfd09321218a01d8aea,
        0x9c1fd673a4f94c9d1ce7dadc8eb8bb99,
    ),
    // atan(62/256)
    FP492::new(
        0x000003cd4291d2dd89ac33591014fbb2,
        0x81a0733bf6daf4bff91cf139179b4011,
        0x1d1fbaa6431498e027783dbdd37edc43,
        0xb6c29de5c3e82a1963ac0590f749d4c4,
    ),
    // atan(63/256)
    FP492::new(
        0x000003dc5c28a3b301999623dcee515b,
        0xd423c91d7dacdb3d6cac62497af5329b,
        0x1e0f30999f94eb28a09adaa5d2e4c7eb,
        0x36fcc70131b7b262ecbdff74f7152232,
    ),
    // atan(64/256)
    FP492::new(
        0x000003eb6ebf25901bac55b71e7bd7de,
        0x885f96a9fea40e22ce0dade8e9d9f251,
        0x269d964ae49459a395d94c16fa316e96,
        0x0c68f8f0af089c1c793e683d070eac0e,
    ),
    // atan(65/256)
    FP492::new(
        0x000003fa7a3f881b7c826e28de7438bd,
        0x4f2b7344f7ea1b5d98d16df4f7199ac3,
        0x47d38ed31c80ed48b6bf7c85b09e5233,
        0x84f63b833470b033a32f8378f9610df5,
    ),
    // atan(66/256)
    FP492::new(
        0x000004097e9441996d698d2097cd05b8,
        0x5a52f129d011dace22cef3eab212e3ba,
        0x76182da5fd8b74bd301ee5522dc95dd2,
        0x6f16781d25d9236fa9167701977b622e,
    ),
    // atan(67/256)
    FP492::new(
        0x000004187ba80f58a43d4113e49d579f,
        0xe16a0ff1fddaf1fd4d38c065fd18f3fe,
        0x7098a7154472a623ab47ed8d8727b0a7,
        0xd5cc4721704448a92b443aba26ad228c,
    ),
    // atan(68/256)
    FP492::new(
        0x000004277165f618d8962e47390cb865,
        0x5e9d1571285505b7e82d8742430ca025,
        0x982282e0b453329fdcf5eb8e6b642233,
        0x656ac3eee457303fd1ec2fa0c0020fb3,
    ),
    // atan(69/256)
    FP492::new(
        0x000004365fb9426b2cd47c317bd5a3ea,
        0xe0bb79af3fd40a05e2e0d7e0be08177c,
        0xa6b77ec1f1489b1b16b22743bdd771ad,
        0xc2f8450642b58cf06beb49ad2f74b596,
    ),
    // atan(70/256)
    FP492::new(
        0x00000445468d890c726b237236f88461,
        0x3c81a979d9a18ba0a243d876d4fea0d5,
        0xe882f54b319bfc0ec8aff65b54ac58a9,
        0x7f939267acb9c46d3bb97a33d4e80363,
    ),
    // atan(71/256)
    FP492::new(
        0x0000045425cea73951a8694580635cd8,
        0x6108b4bf1726d002dbee33f9c4f9548b,
        0x7628b3cd7e9b373a6679d930186b419c,
        0x44f0db9e119e0df3eebe6f9c7fc1f253,
    ),
    // atan(72/256)
    FP492::new(
        0x00000462fd68c2fc5e0986523a458dfc,
        0x414c687e9714de0d27de7e5275dd6a84,
        0x5027428af6d95b8c8226dedf331adae2,
        0x8c34056bac311e76734f49fcc00a83bb,
    ),
    // atan(73/256)
    FP492::new(
        0x00000471cd484b7620f41329c39a6869,
        0x26c0d1131740ef5b9dede8d89bef42de,
        0x95dad74b2522edaf9f7e5229ef96972d,
        0xdb26907ee32832b5c819760a7911650c,
    ),
    // atan(74/256)
    FP492::new(
        0x000004809559f91f25773e6e6b85eb78,
        0xccc11f2adf0aea06064a115f20819451,
        0x46f9cb4e812beb2094c62e0ced958a50,
        0x8fabee6ad44fae00913e9b4a3d2094dd,
    ),
    // atan(75/256)
    FP492::new(
        0x0000048f558ace041078eb3a3a77ffdd,
        0xebe99a0d090cf4564b2aca985fa7c924,
        0x4ac7da7e8b5a61427cc25e7d92cc9398,
        0x4042a8871efaee216bae902a7a010f2c,
    ),
    // atan(76/256)
    FP492::new(
        0x0000049e0dc815fbd16f88322c92037f,
        0x0a23d223e10cf906b19163d78b33a398,
        0x4379f77550ff3d4e2af6c5111774994c,
        0xe672f39a8dac312f70f054c887024f2e,
    ),
    // atan(77/256)
    FP492::new(
        0x000004acbdff66d7f880a11e7e0e16a2,
        0x148aaf4cf1a52adec4ef0bc11d84cfb7,
        0x4bd3d3db4bbcc84402d2d1e2ac1d5f12,
        0x6d3e2b3c577713058331b29a0dfe203d,
    ),
    // atan(78/256)
    FP492::new(
        0x000004bb661ea08f3f8dc892c7500964,
        0xd0de34f9e3ef24c2b9c5ead4a61cce91,
        0xef2eb62a6a1b6d7bf2f67f1156669300,
        0x17bc1fdc857780490a8a70ca9b21f57e,
    ),
    // atan(79/256)
    FP492::new(
        0x000004ca0613ed6254656a0f566c60d0,
        0xdd218a1f7c145d9a44682c7508825609,
        0x9f013c3d2211b6c80d44bde9d1bbcbf5,
        0x3f41a93ea8da998d14a857a1f9e42770,
    ),
    // atan(80/256)
    FP492::new(
        0x000004d89dcdc1faf2f34e2d5da4c693,
        0xd7994045247c28597aafffad2c806098,
        0x263d7699c699254eefbf322033f7a346,
        0x43a2c82674492cea6a96ddd8e02c697b,
    ),
    // atan(81/256)
    FP492::new(
        0x000004e72d3add855eed0b25a5deb8ec,
        0x8b08a1aa6b1c0e8ac86ab782a220bca7,
        0x0edb948467703d985053bc63d814d89c,
        0x0247f240818a3877048de2f11f89f76a,
    ),
    // atan(82/256)
    FP492::new(
        0x000004f5b44a49c44d1137ca41cc9589,
        0xe8ce198e65edf2b34e36a83aa88d9456,
        0x15e281124fb442c8e4f118f40a28f58a,
        0x2ceee588deb893501bc058790271e02d,
    ),
    // atan(83/256)
    FP492::new(
        0x0000050432eb5b1f4ca4f37451ec4aed,
        0xbd671cd697057e2d0c07d55cb0d8f17c,
        0x27be77f10d8bbc80edeaf4c11bf3c492,
        0xe65af27bbd86deeff1be6577638a09e7,
    ),
    // atan(84/256)
    FP492::new(
        0x00000512a90db0abc26a2a1bc3aa4c45,
        0xc6cf1a7413c521a2ec305ed22099be75,
        0x81db1ed9ec1da26cf7aa8cf10222f4f3,
        0xb5bde6c3e76333a4907e5452f7370a29,
    ),
    // atan(85/256)
    FP492::new(
        0x0000052116a1343086d1e28a7563c6a5,
        0x94645cb14d6ea620f7a38c7d9a717142,
        0x7073e105eec8ed3011a20cf591d12cdd,
        0x18d9f79601507d29130503defec716f1,
    ),
    // atan(86/256)
    FP492::new(
        0x0000052f7b961a2439b0d91c41eeac54,
        0x9d5fa0148a6da0866191f1792cda3bf1,
        0x4c37621be8441df3f25c1e67e2aaca4f,
        0xf9e4a1ef1c3c36afb91223557bb3f5e0,
    ),
    // atan(87/256)
    FP492::new(
        0x0000053dd7dce1a65e39a978c2003bbd,
        0xe5d5d587c4cf43089501de5399f1e5b9,
        0x69c3b221a4e2c8a9d28d804c9f360ea7,
        0x1b5a4be7904d762a07664528f518d3d0,
    ),
    // atan(88/256)
    FP492::new(
        0x0000054c2b6654735276d4cdbfbbdfbe,
        0xcf46090961ce98f7a6be9d12e94ea64f,
        0x8f15b707dd4bff930a1bb921dd49b5e6,
        0x1c948daab7c7b619772925ff2e2090e7,
    ),
    // atan(89/256)
    FP492::new(
        0x0000055a762386d335f009cf0c76180e,
        0xc7a655dbb882d344c5a3e30a5e5992d5,
        0x23e88a3db34db6b86a9cda12d0234106,
        0x992ff85532e9b87ab3042d83fd3f025e,
    ),
    // atan(90/256)
    FP492::new(
        0x00000568b805d783d3913b7a8f82e457,
        0x41deb4eda313c6dd4ab0b06eaecdb752,
        0xd7e379c6bc20a189c2701474c9ff150b,
        0x809cffc1d3bda1e7e09401b78b36c6e0,
    ),
    // atan(91/256)
    FP492::new(
        0x00000576f0feef9da34f50760dbb7bec,
        0xe25fb2cb9c490e7595d812d07009350f,
        0x634c4aef6afecb7e3253d2a79d9bf5db,
        0x2c0aac630ba0fb5ad91c66e34c8c8304,
    ),
    // atan(92/256)
    FP492::new(
        0x000005852100c273f8658da8ea8ee100,
        0x507e15a042e6f4bd6b6b817eef8f6850,
        0x05268d7c103001b7c5dedcae5879b194,
        0x0ee0f51b401c35888957e2cac7ee00f7,
    ),
    // atan(93/256)
    FP492::new(
        0x0000059347fd8d7071605bfc183f0242,
        0x309f5848af9637cdbc7b59f5551d0b01,
        0x68cf23d6bd9c16e932e6121121830afa,
        0x68266d851301b6b9a5b844327c217b45,
    ),
    // atan(94/256)
    FP492::new(
        0x000005a165e7d7e9bf7db50fd65ca4ac,
        0x7b49ab58574b2d95a515b4038daa836d,
        0xb9b4fb6029dc54c5613e3e4587223583,
        0xa91b7faf57913ea24f1faee01cb6c6f8,
    ),
    // atan(95/256)
    FP492::new(
        0x000005af7ab272f5db3968ea706485f6,
        0x062fa576944aca1ec768f7711076ddc5,
        0x49a6b12e22eaf251beac4dfd7106b8c2,
        0xa57a048efa71bb71f178b4287a50cd91,
    ),
    // atan(96/256)
    FP492::new(
        0x000005bd86507937bc239c55190916e7,
        0xf22419ec21cbbd72a2ae62399f2e519a,
        0x4847032662c101cfee06b6e667254006,
        0x7b9e9430b7703bd7d0d30243ead569a5,
    ),
    // atan(97/256)
    FP492::new(
        0x000005cb88b54ea8aa62636cb1860d84,
        0x315183df5116dbf97b157596df9e5cba,
        0xa36b1938f993f57743f3d440cc956363,
        0x21b87c32a6518701bc5c5e6ccbf07150,
    ),
    // atan(98/256)
    FP492::new(
        0x000005d981d4a05d407c45dd0dc6713f,
        0xffa13e3d8a98698a666915f8ad04e8ab,
        0x750c5aec2918af43782dcdadffa83967,
        0xaf23b726767e2453bc84d625346c429e,
    ),
    // atan(99/256)
    FP492::new(
        0x000005e771a264463440dba09d077b03,
        0x66780e2ad83c474be0aa17b4d1f80e95,
        0xb8ed45a0c8c03fcb1fe12c7a63e5a067,
        0xf1ca51feba9572cf0829350a00100b7c,
    ),
    // atan(100/256)
    FP492::new(
        0x000005f55812d8ecfdd69c885c2b249a,
        0x0881312e09e0eaf2efb9fcb1fbef21e7,
        0x4ec5920529daabcd20a6447d65dfc894,
        0x78ccbba32611734fd2cc9714c74d2476,
    ),
    // atan(101/256)
    FP492::new(
        0x00000603351a852c74218606697b05e6,
        0xcc5ec618c28f8372f9424909b7c56954,
        0x605754cbfbbfcc9b180294c4a9cb9836,
        0x2d5a59d532daceaca59cd38c59383476,
    ),
    // atan(102/256)
    FP492::new(
        0x0000061108ae37e575dd76a0299b41b5,
        0xc3a3b8175a01c88d4dc132aa1a59a09e,
        0x11aa8ffe7ddc0984107aec6eda544dfb,
        0x5d55f76a48da9e3e4ad236a7d6ee7846,
    ),
    // atan(103/256)
    FP492::new(
        0x0000061ed2c307afb6e923055d05b8f0,
        0xb9465d580574f77386fda67335fd6fdc,
        0x2df8f6e4b5b4be4d8d932c1604fc3199,
        0xf972ea764abac99972886c6b20f39d02,
    ),
    // atan(104/256)
    FP492::new(
        0x0000062c934e5286c95b6d0ba3748fa8,
        0x5146ee25be4f2869d50fb413ddca5d84,
        0x9a43ee3821dd43a18f632a348ef4c9f1,
        0x25b1dbe201df29b5c63e5eb9e214cb3f,
    ),
    // atan(105/256)
    FP492::new(
        0x0000063a4a45bd737a14c0a7e12bfafa,
        0xf4343ee0dd669eb360681d2cde838492,
        0x01fe805873f7a6b6b5a939569857824d,
        0x8b216d7a15f8efcc501fb8c26ef59d3b,
    ),
    // atan(106/256)
    FP492::new(
        0x00000647f79f34319891074188054b53,
        0x6bec637d321248b92df5b37a7a057c0b,
        0xd6eb79f56e7a5985febcd23d487c1680,
        0x6ec4758b65e0477406e1d27cc0e8baf8,
    ),
    // atan(107/256)
    FP492::new(
        0x000006559b50e8d241ccd80a9bf05795,
        0xaf8a9973e149e68afe4fe7c311a23ddb,
        0x0a5c74595f087368062abb859bebc2ef,
        0x8bd4b9832b084c3147f3d7e0ba8e5c4a,
    ),
    // atan(108/256)
    FP492::new(
        0x000006633551535ac619e6c988fd0a76,
        0xcdbe1c93d002a4410cb0a0991b3376de,
        0xa2a481033627d11a5a672ae70741dbda,
        0x9e833860ab793f9413e9d2d503b1da7a,
    ),
    // atan(109/256)
    FP492::new(
        0x00000670c597316041c36cd2c044a19f,
        0x5325f29c10727ef24abd846f78781106,
        0xf2c728178c086ef5e863f9fc246c83f8,
        0xba4fdb07edd48dfec5842c9afb77874c,
    ),
    // atan(110/256)
    FP492::new(
        0x0000067e4c1985a000637d8fb836bc65,
        0x81a7b995f498a03790b35847c5e96044,
        0x7819e8ab8886ef6c1005cf5eb5b4e71e,
        0x79e2055afa598f3fc02a748b7ba241b5,
    ),
    // atan(111/256)
    FP492::new(
        0x0000068bc8cf9794c2c605357de5c21a,
        0xc153c672881904831b49186690fdbddb,
        0xe058ec5f7dde1ed5d01f4fee46f9faac,
        0x5895559c264aefc57391c68562ed4e77,
    ),
    // atan(112/256)
    FP492::new(
        0x000006993bb0f308ff2db213e4af4800,
        0xf389b3700206e90b0d39e1333bfc789e,
        0xebf14142bf4205e2241da7bcb42c929b,
        0xfb62e1b924c6ddd66ae64e1150332d70,
    ),
    // atan(113/256)
    FP492::new(
        0x000006a6a4b567a633c25c85bfeebb02,
        0xd4a70c3f91d5ac0d45d26bcc0fc99780,
        0x8827b4077892917dc58594a6c6b430a5,
        0x89d303d6c4d162defd0b07b8086fb94c,
    ),
    // atan(114/256)
    FP492::new(
        0x000006b403d5088162dfc4c33891d2e7,
        0xeef7e0a43580c014ae9c2f93f9ea2e3d,
        0x8c4f6900ae95db881b6a8d073f53162e,
        0xe5c79ea7accca5ed2e7940e0a6de9c32,
    ),
    // atan(115/256)
    FP492::new(
        0x000006c159082ba4d0e6cec6b9c9cbaa,
        0xa6baeff35aaf24d7b491a6f139339073,
        0xec382ddca2001cb1ccde1f04ae75291f,
        0x8c2b8615f314fb90c6c1a0f2a8783437,
    ),
    // atan(116/256)
    FP492::new(
        0x000006cea44769971b1ae187b1ca5040,
        0x31a2eaaa4088c5fdb82266469134faca,
        0x1b3aaf0b41878e11bb506d88cfb0ab88,
        0x18fe512095bb5a4926cd7427ecd17dd2,
    ),
    // atan(117/256)
    FP492::new(
        0x000006dbe58b9cdfbef8d6b2e6d9f863,
        0xd50c8cd7949f866ff409ee9c8d9f85d0,
        0x457a41362259f8f7b787f8ff03f2a738,
        0x05eecff1590c95bd87952e32fc2a6f77,
    ),
    // atan(118/256)
    FP492::new(
        0x000006e91ccde189295519a1b46e4afe,
        0x929cec1897ca0d14d735319d62e63aea,
        0x48300c6adc4c4bf4d1599b0710485453,
        0x52e49170485e9cde99b80c4e1a3b40a7,
    ),
    // atan(119/256)
    FP492::new(
        0x000006f64a0794a0646b4eabc6509499,
        0xb66209b4bded173fc6ae7e8c8fa9b585,
        0xf6cf44bab502e1cd9af6c1a07fadf422,
        0x0370a5b694f4c7c87e0719dfb0b22639,
    ),
    // atan(120/256)
    FP492::new(
        0x000007036d3253b27be33e318f6cb3cc,
        0x65c01db0a5f97af9f5c11ca859f5e2c3,
        0x2a5bd56ab8ad21a4d44984fb13489856,
        0x51cfd6b9471465449ded6519673e749f,
    ),
    // atan(121/256)
    FP492::new(
        0x000007108647fc47ada4e820bcb4c112,
        0x3a70868db9c1ef5c2a63611f68129808,
        0x0388cdb58ee6043250ba3a7ea6eebd27,
        0xe2dc43d7f72ab8730b4c89239d4014a6,
    ),
    // atan(122/256)
    FP492::new(
        0x0000071d9542ab5c7e28b474008fbc1b,
        0x87ea7bf8b16e66ae7e565d35aa34ae9a,
        0x086586bea56726b31ca451f808d6e77b,
        0xc6aeb4c4a28ae658660c8bd5631c6433,
    ),
    // atan(123/256)
    FP492::new(
        0x0000072a9a1cbcd8c6c0ca833c0b1b7a,
        0x3f6a75619398598792e643e38f1526a1,
        0x8dbad7e21e4c2122e4bb8fff14938192,
        0x27beda879f9b2bc4abe7cd416db0682e,
    ),
    // atan(124/256)
    FP492::new(
        0x0000073794d0cb04d425d305bbe70e53,
        0x6e164325927439e7941da0581e081a0f,
        0x59d94eb573dbae08b7c3d33d0ce8752c,
        0x38f0ba83eebfa920b5c3c91743da4ccf,
    ),
    // atan(125/256)
    FP492::new(
        0x000007448559adfcab5be67835886c30,
        0x2bbadcebb3f7ce05d438eaeeb3ba58ff,
        0x60eb03b05f6407f5291ecb32ebc64978,
        0xef7b73bb34acf92403ebec78f7735ddd,
    ),
    // atan(126/256)
    FP492::new(
        0x000007516bb27b218acc4a108b1f7c96,
        0x9a4a369ae94166ac5248f9b8814c143e,
        0x8a4a729ed1da8535b540d41da4ad68e8,
        0x9004f5c3958e97ab24ad44d9559386f7,
    ),
    // atan(127/256)
    FP492::new(
        0x0000075e47d68489bd35ff40ad24dac4,
        0xfde79cb45b4d2dffeffc8f1615af401d,
        0x2b55946fc7a78e3dd84e12a0d869418e,
        0x3b13957fee8e507bb1dd2195d1f4fc9b,
    ),
    // atan(128/256)
    FP492::new(
        0x0000076b19c1586ed3da2b7f222f65e1,
        0xd4681b70a0ac3930e6f8071678b7374b,
        0x12384fd4e2c8bc495a8b643e4097c635,
        0x230c16770f4077e9e0009eb6c2f1b431,
    ),
    // atan(129/256)
    FP492::new(
        0x00000777e16ec09a5d0a1ddd86050912,
        0xe14cb66efb53083071db295bbedffaf1,
        0x9971a859211419bf339eb0af8dc1c402,
        0xf4c8fda84ef400907c7f19e0ec0efe66,
    ),
    // atan(130/256)
    FP492::new(
        0x000007849edac1d12bfb53d9e719cad5,
        0x5185bc787457ef5fe07640c7167a74a4,
        0xbc94f43207a63b37db4f4173f4d3e694,
        0xaf72c9d5d24bac0543a9e840615638f3,
    ),
    // atan(131/256)
    FP492::new(
        0x0000079152019b3d468274754259d31e,
        0x3ba15b11e90da15d79e70f8f291874b0,
        0x5512099b0e98a7357a85309c4de93bb5,
        0xb14fd961ebfd580f03ecc5313f7c823c,
    ),
    // atan(132/256)
    FP492::new(
        0x0000079dfadfc5d68d10e53dc1bf3435,
        0x6f9fd1790505c402ec723eca3443d27e,
        0x899fff87a0d00cf53d4099d0680bc937,
        0x1351504ecd0e8cbe30ddc6ba57a5c3e7,
    ),
    // atan(133/256)
    FP492::new(
        0x000007aa9971f3ca31097d967651df82,
        0xb549a759edd0f0030dbc82fb0684380c,
        0x7e3e42082bcc00191dd6baedbe52f8a1,
        0x42bce65bbadf1335808695474f21df87,
    ),
    // atan(134/256)
    FP492::new(
        0x000007b72db50fe10d380da2733ddec7,
        0x1326e2c085d461b2d1a22c806eceed98,
        0x59493d9dfa295118640d9ea24fe10ee4,
        0x52cb355ae1bdf1adbf4daa8999886e3a,
    ),
    // atan(135/256)
    FP492::new(
        0x000007c3b7a63ce4f3ed0bf57834e76b,
        0xbe5183efac40fffe98dfadbf97fce7dc,
        0x1b9f209191e5831892a2ed14dd4f5047,
        0xf8b07852f89238275ef25a9a1152e383,
    ),
    // atan(136/256)
    FP492::new(
        0x000007d03742d50505f2e33691e3eaee,
        0x476610806496fc5c5aac1b190087d090,
        0x4133566ce6be148c26c4312df65ecc35,
        0xe080ec83a92d8aab03c7d215b810fa6e,
    ),
    // atan(137/256)
    FP492::new(
        0x000007dcac8869392545227083dfadd0,
        0x04a57b9b4c8a2f26bdfdca47f7e70ff5,
        0x371ffcd00a912d7aa375f7ee70918dcf,
        0x4e3d96cd9771f9c52349bb14dcbb7b5b,
    ),
    // atan(138/256)
    FP492::new(
        0x000007e91774c0a496235a9c23d7274b,
        0x5310ee6b4f66cb5412874261a128e990,
        0x73e719db942af4d9894be14e05ec1e38,
        0x5fc828209855398887779e55039228a7,
    ),
    // atan(139/256)
    FP492::new(
        0x000007f57805d7f7e0c9d43d8f624d8a,
        0x9d6729f93b5a82228a70317f74800fed,
        0xa39671aaba8ac261b465b8b2b9c42598,
        0xa7168ab044b99c3503ee8722959ceb50,
    ),
    // atan(140/256)
    FP492::new(
        0x00000801ce39e0d205c99a6d6c6c54d9,
        0x38596692486326fe2e1cc02f253ef962,
        0x0b8c8434e6df2a7a15c3b764e5cb1c45,
        0x5de90c30512490257e1741a64cb224f0,
    ),
    // atan(141/256)
    FP492::new(
        0x0000080e1a0f412116a7b689f99521db,
        0xb92b1ad73c70829b081740fd151a6bc1,
        0x99b543109b636cc9f220c78f852506bc,
        0x2b670f39ef51125cddb51a680ef4a077,
    ),
    // atan(142/256)
    FP492::new(
        0x0000081a5b8492824418f9b38d85540f,
        0xf5431c9474f20630bb75c3123f517e55,
        0xe07815f3b5ceb4db415e9db10d86c02e,
        0xf04a3527a6c07d2f219328c1d097b841,
    ),
    // atan(143/256)
    FP492::new(
        0x000008269298a1a172dc6f39377a7131,
        0xd6a27bbff43ff88082c6d3c042abe6ed,
        0x19a9ca76401d8510f33bd26ab519b192,
        0x7739b82fdded3d15d449bb622442b39b,
    ),
    // atan(144/256)
    FP492::new(
        0x00000832bf4a6d9867e2a4b6a09cb61a,
        0x515c0f1155cd8774ddfbc55c6bdcf1e5,
        0xb65d043d7d60cd4f13f4e82c3bd5bd4b,
        0xddc248e2871f553ff817d6ff1ea97f43,
    ),
    // atan(145/256)
    FP492::new(
        0x0000083ee199274d9c1b70c9e04450ac,
        0x73920c96ba6dc797b9e09cd470655e2e,
        0x7ff896051dd2530d93e05546155975a1,
        0x07ad8147fc691482aba2fe1fbcb45639,
    ),
    // atan(146/256)
    FP492::new(
        0x0000084af98430d2c7eede4df5ea560d,
        0x2aaeaaea472105c56d4c59e3f429f91a,
        0x471e22a95ee246880ee04d49a7e5323c,
        0x448e8949a98ec12a339b92e18760d709,
    ),
    // atan(147/256)
    FP492::new(
        0x00000857070b1cc3361075abf2de445a,
        0x53e2c75d49076c037a18272b5a0734e7,
        0x886d0ba3ed0a598005332ab8012f9cb9,
        0x3c484a7123346551f7fa489fb68ab6bc,
    ),
    // atan(148/256)
    FP492::new(
        0x000008630a2dada1ed065d3e84ed5013,
        0xca37d92a950da94553290ae8bed899cf,
        0x54cc745f546519bd67954c112f51cb87,
        0xb136bf6a5b3b9ebac14430993b796b95,
    ),
    // atan(149/256)
    FP492::new(
        0x0000086f02ebd537be67c97b4e3ad655,
        0x622ba7c75090d33c397b1208981dc7e4,
        0xa61d5c8e74cc91ddad3fb50ce16acf40,
        0x586d3e4ae36b3fd5036dff66afd88cc7,
    ),
    // atan(150/256)
    FP492::new(
        0x0000087af145b3f14a800988d8bf64c8,
        0x681429bdd6bf9f150f32c2dea9c231f4,
        0x28f58a2a7f4f882e42791d6dd6a02330,
        0x69252badcfeb33d4678f4a1a5dd2f55f,
    ),
    // atan(151/256)
    FP492::new(
        0x00000886d53b983d06ad3867a898f60a,
        0xf9f5dbce1325f98c02cf7c7b587391a5,
        0xe9ca19d41053937fe5a4dd0d0bc8ea66,
        0x36083b3c1550b45f2d54321e5977c241,
    ),
    // atan(152/256)
    FP492::new(
        0x00000892aecdfde9547b5094478fc472,
        0xb4afb8fbe7b9fb9ddf67f28c0a22e65f,
        0xf600e31205dd0e396f760a2d08b4adfa,
        0x2ae57f42337e8b2b51bdd998d73c4edb,
    ),
    // atan(153/256)
    FP492::new(
        0x0000089e7dfd8d82b726221cdf87da6f,
        0x989c0b2e59eefaa77cc19f400d02fa75,
        0xb251f075ea1ea3712644cf607f342e78,
        0x42a54ff14350ae4b281a5d43f64b0c7e,
    ),
    // atan(154/256)
    FP492::new(
        0x000008aa42cb1bb234d68b0756e81879,
        0xe8439fe76f7eccfbbf3d6b5fc0254dc4,
        0xa203b458196e9be81d9a526225539c96,
        0x7a295f2d8d29ba97d05351249935476f,
    ),
    // atan(155/256)
    FP492::new(
        0x000008b5fd37a89bf099606fe141bd34,
        0x8634d99492cf6ba02bf4d3f549952941,
        0x0115c4d37e8ad9c06b113299f56c09cb,
        0x81009b4b5656436ae3e59470d80d1c78,
    ),
    // atan(156/256)
    FP492::new(
        0x000008c1ad445f3e09b8c439d8018602,
        0x05920f8e244490311ce06ca922e3e403,
        0x64e6fb8340b68a4a934c22a2e149b88a,
        0xd45d8c7dd9b3f5ff1e675c4e88cbc23f,
    ),
    // atan(157/256)
    FP492::new(
        0x000008cd52f294cfcccb3f27a74e0f96,
        0x02b3b8709d0f2148f8aa62cdaf3be86d,
        0xca7f6082c4f3d40643c3204c22fd818f,
        0x0d4702ae68edeccbc2484943f5994508,
    ),
    // atan(158/256)
    FP492::new(
        0x000008d8ee43c8214276f0b9ba88386b,
        0x8a2ccdfc2b8615cb6dba333d3ce86493,
        0x8f3868e0c22bde44976ce78a9fffb120,
        0x13d43c488b568c877ab709f183483cda,
    ),
    // atan(159/256)
    FP492::new(
        0x000008e47f39a0fb27928bbc9d5e7929,
        0xf8ecdef9cec1a885e59ba94ca76369a9,
        0x89102adf93983bf30f35a94ea9a9e6b9,
        0x2045cb7ed34dd9d86e4e7b1171923808,
    ),
    // atan(160/256)
    FP492::new(
        0x000008f005d5ef7f59f9b5c835e1665c,
        0x43747918a67e0652b375cf53da46d133,
        0x89eb23669dcd3918d712b66cd7dd3073,
        0xd6a5829c4ed1d7e523acc358f6440a43,
    ),
    // atan(161/256)
    FP492::new(
        0x000008fb821aab89c516c659f6d7dd47,
        0x136a9b668d8770e1ca0a26573867f42a,
        0xa3f89b33adb8f31636f6c3058e42f160,
        0x3e75fdd2e444f9042f128e8e42d7c63e,
    ),
    // atan(162/256)
    FP492::new(
        0x00000906f409f411d8d0dc5158289178,
        0x50ab599f0a484419afea6fd954e7283f,
        0x80434d44568eff0d6ffa99b146e4ed8f,
        0xf20fc1c8982a54e6ce0660e32ee21ce0,
    ),
    // atan(163/256)
    FP492::new(
        0x000009125ba60e8c953ada7bf114c188,
        0x0f6a2b0b51d132051239d25831acb28b,
        0xd151e9f4f2fe10c778e51af657582a3d,
        0xa5238ec0fe7593cdac35e8b370708e82,
    ),
    // atan(164/256)
    FP492::new(
        0x0000091db8f1664f350e210e4f9c1126,
        0xe021fd995e8d1fc3534376758f20e06e,
        0xfbe97684ccd8dee48c3903ce543133eb,
        0x0fb423994dadf2266c6909f6e7953eda,
    ),
    // atan(165/256)
    FP492::new(
        0x000009290bee8bf280aad5511836cf37,
        0x97d76ea50b3e721bcbf2d24b7bedc86d,
        0xc7ba37dde84bc0716dec0f1315802eb8,
        0xa284b2f3faad1df0f70f9027094cded9,
    ),
    // atan(166/256)
    FP492::new(
        0x0000093454a034b6d30744d228131cf3,
        0x1c981c4db4f92c291b97daeaad9a7fee,
        0xb773fbedd38a633e3fa3e303e84683ae,
        0x6bac9de6c8285a1106e41222db398bcf,
    ),
    // atan(167/256)
    FP492::new(
        0x0000093f930939e8d9a877e6c2a3bb3e,
        0x2ecda573dc02ff77c6d78d6a436474af,
        0x2fbc42bfa01d60275deba78fe9f00074,
        0x7ee2cb4a83e2f0a1ede39e919b46a4aa,
    ),
    // atan(168/256)
    FP492::new(
        0x0000094ac72c9847186f618c4f393f78,
        0xa32f8f38ae0f47a945eda2c6b9f78803,
        0x1c786366dbce1b97e806e892aebb8234,
        0x2e027307f92239fc8cdd23ea57fe00f2,
    ),
    // atan(169/256)
    FP492::new(
        0x00000955f10d6f6839b85180035a045c,
        0x3b111748dbe4700ec6abf6e81047d6c5,
        0x50ef3535e07303f5b6c6c5faac9053f6,
        0x257a17d4977f607e2277e02013b6763d,
    ),
    // atan(170/256)
    FP492::new(
        0x0000096110af012232fd68fde4b8b683,
        0xe268649af3b91948783662595a117e59,
        0xcc3c64fb92fff3e6ac2a317d7f06f72d,
        0x045fa51d623a665b5cffa57075bdba30,
    ),
    // atan(171/256)
    FP492::new(
        0x0000096c2614b0f245e0dabdf5c91afa,
        0x42aa81fd293f462212feefbc20be4ee5,
        0xae7c504d4dfe807b54f2d9654eff2de7,
        0x8a0a8b8117ff74da617827b980db20d3,
    ),
    // atan(172/256)
    FP492::new(
        0x0000097731420365e538babd3fe19f1a,
        0xeb6b29798db274070578e1faf20aa874,
        0xc64d423b240c5b871bf60048b1a46cc9,
        0x212148173f3b2b6e2a67815f83a6aac4,
    ),
    // atan(173/256)
    FP492::new(
        0x00000982323a9d84856c18ed9e25e70c,
        0xdba0ed14fc58264bdeff81481e51fb81,
        0xde37c66cb95ffa572d976cbd555f942a,
        0xd7290986431aa1683391e19b8255c4d4,
    ),
    // atan(174/256)
    FP492::new(
        0x0000098d2902443a5f2819570f4f5848,
        0x92a42fdcc108f999fc0da25f3e25b932,
        0x4c0a084a48c058b48367551c3add786d,
        0x83c220efeead3074e1526d8a3ab60efb,
    ),
    // atan(175/256)
    FP492::new(
        0x00000998159cdbc42b2bba5db0f75056,
        0x78ad6593e8d1b40b7b8afd521c6db82d,
        0x42a04d675d8eda9c5d9f020c62241422,
        0xcd5790ac0a9b1183f9d16ad5594ba4de,
    ),
    // atan(176/256)
    FP492::new(
        0x000009a2f80e671bdda204226f8e2204,
        0xff3bcdae46f0617489d5c77874d1e753,
        0xafee44ccdbfe6b74ad88f11620ae2966,
        0xbda9c098e55dd9c44fa81e9e70d29141,
    ),
    // atan(177/256)
    FP492::new(
        0x000009add05b0766673c77433726d172,
        0xb70996c03d1d89d7f9d9b643b549743e,
        0xaeb9ac496ecf6b4e2ea0eaad9c2420a1,
        0x67aca218780aeb60a89e6d9ec50cffff,
    ),
    // atan(178/256)
    FP492::new(
        0x000009b89e86fb6281fac21d87f3a7e4,
        0xb6dd3c5daac0edab7679ae01aa73425c,
        0x955300aa57e64a60d5850e221dcf091d,
        0x4f19adcef5ee2ca15bc11ee73c1d8cd5,
    ),
    // atan(179/256)
    FP492::new(
        0x000009c362969ed88f490e3c682832bf,
        0xbda16e92930016001e35eacbee3501d4,
        0x3e268c5d140364cdf5cc3581ea5950bc,
        0x7c35303cc74818891df0e2f36417d35a,
    ),
    // atan(180/256)
    FP492::new(
        0x000009ce1c8e6a0b8cdb9f799c4e8174,
        0xcf11c5a2ac6a3b26e793ef1e5725d60d,
        0x2cff2c5acdb526cacdafa34d63cadd72,
        0x640eaf026319fa36cef1ea95e1a5a253,
    ),
    // atan(181/256)
    FP492::new(
        0x000009d8cc72f12b256d06dde0f8ad54,
        0x20d93feebfb012e5585bbeb87c63b5ef,
        0x33dec61ea308cda8196d586e84f415ab,
        0x03b8ea9998dd2b80a576bcad840e0f45,
    ),
    // atan(182/256)
    FP492::new(
        0x000009e37248e3c6e243d75ed605e8b1,
        0x28b9c394f7ac857c9913e8c9ee65d54b,
        0x2ce86161fda6a360e6a2812a703774b2,
        0x226f2ecb6accb6d950e93b7713e78de8,
    ),
    // atan(183/256)
    FP492::new(
        0x000009ee0e150c4282259cc6e1e9d2e1,
        0xdfdf5925a09fffd8b9437af09eda9516,
        0x31e669141cd380d8b79ad875ccc16a52,
        0x36aaf149bdb56e3fc4e25664860bd690,
    ),
    // atan(184/256)
    FP492::new(
        0x000009f89fdc4f4b7a1ecf8b492644f0,
        0x701df9d743d1bc801acaa00a35bc21c6,
        0xf4dfbdb76883077858f462837a0dd676,
        0x74a8b4a8dcdfbed899dbd149416e92c6,
    ),
    // atan(185/256)
    FP492::new(
        0x00000a0327a3ab4fa44aa4ee0dc81a9b,
        0xe5a70912c563a99ca96f1570dac4cff0,
        0x55c4999215509850b93767c4390aef1d,
        0xd3e6bde76ccd34e31a5f0a8bcb9cc7c2,
    ),
    // atan(186/256)
    FP492::new(
        0x00000a0da57037f52089eb052993bb28,
        0xec56839662ecbf7c6370433bc2432ad2,
        0xe8823dfe1392e5d2c3b833a9aa868524,
        0xc029b70db77e1be0760ad0104baa5280,
    ),
    // atan(187/256)
    FP492::new(
        0x00000a18194725936add9d4ddf1e8cc2,
        0xef763f0f53593017e39321031ccfe85a,
        0xb70c7c54d6eb8ab6b4b529a50d136a04,
        0xb082060a29d7d104858ef89642133dfc,
    ),
    // atan(188/256)
    FP492::new(
        0x00000a22832dbcadaae0892fe9c08637,
        0xaf0e5d084146d4fd55be415a11f3b502,
        0x2d783345185e92cfe1d33f1bc281ee6e,
        0x9150996b02c2a0513d49b7a2fec8c98e,
    ),
    // atan(189/256)
    FP492::new(
        0x00000a2ce3295d6e3fa332d31d1c26b7,
        0xa893d3f56d575ecbcdf0a55753016548,
        0x9905ab2a39eb0b7c7b50f770208acf23,
        0x73a5b4f87f3e94630d84ed181efdadb2,
    ),
    // atan(190/256)
    FP492::new(
        0x00000a37393f7f238af63232723dd99b,
        0x9efd38377a1ec574756daf1fdd467151,
        0x9b7d42363a23f4c8045a6a25423e9b40,
        0x8341a6b338fd3353878c0481b5a55861,
    ),
    // atan(191/256)
    FP492::new(
        0x00000a418575afbdfef9766a66626d63,
        0x334d8d43b4e063a8e33afa6f75e8b677,
        0x7e14a1f8c1678375987356935ca952af,
        0xa72fed1652a5451157fbb2f3c35f4e2c,
    ),
    // atan(192/256)
    FP492::new(
        0x00000a4bc7d1934f7092419a87f2a457,
        0xdac9ee3f08689eeb2b9e7214866658cc,
        0x4ef3aa7f7b7db933cb84f5762206ed3d,
        0x024b391742ccce782285ac8ea0ca4548,
    ),
    // atan(193/256)
    FP492::new(
        0x00000a560058e38bb13641ce47057200,
        0x3eb4629ef1eca0cd0e58d9b1cffb5cdd,
        0xd11604a9046c5e9548892e09b33429bf,
        0x6f3e3b69b5678b357f88c4ebb7d100f8,
    ),
    // atan(194/256)
    FP492::new(
        0x00000a602f116f4a7247ea7c1ec1a242,
        0x451d4464264c5c916e8e8ec759243d72,
        0xf6e5526d2e613004761ab45f44fadd46,
        0xcd0be3ebf15021f72af1a9f506f83817,
    ),
    // atan(195/256)
    FP492::new(
        0x00000a6a54011a0a740f1d448ed54458,
        0x0ee24a1872a95c2ed13cb3318a32b115,
        0x2656dcc9b9d5d135455621ef1ec9383d,
        0x36f125b029b160e0c37546068a970c26,
    ),
    // atan(196/256)
    FP492::new(
        0x00000a746f2ddb760229467b7d66f2d7,
        0x4e01921b81774d87a36a4eb3fe5fa494,
        0xa132239c4080f6ddbb82509f6434092c,
        0xf61f4f385b0baa8eee07245f459439c2,
    ),
    // atan(197/256)
    FP492::new(
        0x00000a7e809dbee8bf1d513f3e7c24b4,
        0xd1412b0d212cae235aaba566e7e8bf0d,
        0x11959e9c2ff7ddcbda61dc43078b512d,
        0x4e3b841da709b83cbe1ad6ac8c4f7cf3,
    ),
    // atan(198/256)
    FP492::new(
        0x00000a888856e2f6c0923dcd6832a63d,
        0xe1ee8e17ab4a5af4bc660878211b1f25,
        0x7d01eeca1ba5d448c4b559ede3d4c247,
        0xe47ffdd83543905d79a4b9c76be14e2c,
    ),
    // atan(199/256)
    FP492::new(
        0x00000a92865f78f4fd7ab19217375de4,
        0x1b9df73ca99a3527462b0bdbab4d9b62,
        0x709d6a88d2a6df528f87f28bfc71abec,
        0x24271d998cbba1e72816dd1ea5a0ef98,
    ),
    // atan(200/256)
    FP492::new(
        0x00000a9c7abdc4830f5c8916a84b5be7,
        0x933f5f9971655e427bf1c094f003a7db,
        0xf5c4d45931d7436a1ca19ff7ad99b056,
        0x2adec72eb259ff8431ede85c67da6061,
    ),
    // atan(201/256)
    FP492::new(
        0x00000aa665781b1647b14648ff1e8782,
        0xacc5172cde54877ef508b91bd6827f76,
        0x9d388ec29e77a102e3539abc4e90acfa,
        0x4716dd4607a6b79d2847ed84d5aeb2ee,
    ),
    // atan(202/256)
    FP492::new(
        0x00000ab04694e3861a332739e1bd0110,
        0x00fd4fa1db9e78e69e2709f321fca5bc,
        0xa35857fa5fb8ae2ec9b6d286ac91b0e2,
        0xc01519a2542b288b7fd78cab1b0aa633,
    ),
    // atan(203/256)
    FP492::new(
        0x00000aba1e1a9599dcc2bfe5966f02b4,
        0x69fb603728b70e0d2c55b56a206202a4,
        0xd08e414ff539ce73fb9c1b40008b7ba4,
        0x85390cf855bad618b50ba400185f2c14,
    ),
    // atan(204/256)
    FP492::new(
        0x00000ac3ec0fb997dd6a1a36273a56af,
        0xa8ef4183db5406c42068cb854b5cfa7e,
        0xdf0553cc6eb351ca234ff9b9aabd16f3,
        0x25ae34a195dafc3bde77fd3fdb776fd9,
    ),
    // atan(205/256)
    FP492::new(
        0x00000acdb07ae7d5cedc9f677dfb2876,
        0x9f24d1951bb83130dd772f4309a24c4e,
        0x2513f648152169defb54cb278542a39a,
        0x2b99ff90a8bfdffed0b204614067c36a,
    ),
    // atan(206/256)
    FP492::new(
        0x00000ad76b62c84a8bae625ee295286c,
        0xdb591529f4f435ac7f639361bd73d327,
        0x860b82041007364684b8f8d0d947ac10,
        0x9498e7bb25f2e593da3a953a309f46fc,
    ),
    // atan(207/256)
    FP492::new(
        0x00000ae11cce12213059c8562c352995,
        0x154abc1476866c42ac10401e2dca4c5f,
        0x182b71b7384afa482e425813b0f959c8,
        0x9b025c068fdf710ced33d4c5681e842e,
    ),
    // atan(208/256)
    FP492::new(
        0x00000aeac4c38b4d8c08014725e2f3e5,
        0x2070a03742b4643effe2604407947c44,
        0xfdd3095ee0fd1eef1f3d7c6586fe1dd4,
        0x39af33fd6f7853b746c2d5307ed4bc54,
    ),
    // atan(209/256)
    FP492::new(
        0x00000af4634a0821e7ef408a41bae190,
        0x4f9164736fc44bfbb5c686dd5e77f4e3,
        0xa097c77bd59a9b6b5a68c3843bdcb43d,
        0x8a1f6778f23a9608d99e1069518ef19c,
    ),
    // atan(210/256)
    FP492::new(
        0x00000afdf8686ae624f92cc0bffd23aa,
        0x0ae627e411319f25347b4922e6f31728,
        0x10af72a7b1f70916f0675f7f6847324d,
        0x12ebdb16df078ef9a241acbf0d7450e6,
    ),
    // atan(211/256)
    FP492::new(
        0x00000b078425a3702f4488f2ccd75cd9,
        0xcdaf2a5b0f42a1149f51704696a5f13d,
        0x626cd1ddf854de64810a7a5f8fbc474c,
        0xf6262c74448b53fcf63dfee29a6a0ec6,
    ),
    // atan(212/256)
    FP492::new(
        0x00000b110688aebdc6f6a43d65788b9f,
        0x6a7b509e2828d4df9e1c75d3ed56bcce,
        0x6db2b205110bd042594fc790bb09abdf,
        0xca441314716be119ac76f3a0ebf2d220,
    ),
    // atan(213/256)
    FP492::new(
        0x00000b1a7f98968f9db3a5e62be56b2f,
        0x9e94a3ae3c1f6716ea56a66a635d2d5f,
        0x02eee2e4348852db1af6306202b00d28,
        0x24dd152abf343b70a6d35499d0c17ea6,
    ),
    // atan(214/256)
    FP492::new(
        0x00000b23ef5c7105c7f84d7eff716b8c,
        0x33242b2a3bdca53734cea3c3177a2c0f,
        0x40bad25329c7984ef727839cb4dbb5fe,
        0x4e0c5ead0314f6c3402d31a366759504,
    ),
    // atan(215/256)
    FP492::new(
        0x00000b2d55db603d8174363262866c6b,
        0x9dc4ba2d0847e9071ce0eea469bc38a0,
        0x944eddfe5b79263e3ba8fbdfba59ec7c,
        0x3f6c2c63c589bfc5647397763ad684d8,
    ),
    // atan(216/256)
    FP492::new(
        0x00000b36b31c91f04369159014174446,
        0x2f939e469ff280783f6fe751e5dec409,
        0xdcd1e43ac8c5fef498a5770c045a9fca,
        0x71d8c0cfdc69c9e977afb4785f476d04,
    ),
    // atan(217/256)
    FP492::new(
        0x00000b4007273f142bf9c56579305bc3,
        0xf157669a530952ffd0ff743c06970ea3,
        0x7fc5fa6e1ef9d8c44cea375f0e0087dd,
        0x4e84f3d2abd15a073dcc12238c933c07,
    ),
    // atan(218/256)
    FP492::new(
        0x00000b495202ab7db53b2972b51d4597,
        0x1ca787426f3bb8e5b3e750a83bf69551,
        0xc43567486ae524359fdcf0c2b1434cc4,
        0x70fe03c2d947889caa556d3fdabd0c02,
    ),
    // atan(219/256)
    FP492::new(
        0x00000b5293b62582bac12934c772b2c3,
        0xf989bc0998af1fab778f97c6c5a1001b,
        0xbffe261ae10f0ff66eee0f453dc7b89f,
        0x6a7009a25eeaac5632729a308e1fbb6b,
    ),
    // atan(220/256)
    FP492::new(
        0x00000b5bcc49059ecc4aff8f3cee75e3,
        0x907d575216f47b3891e078cddaac18ce,
        0xa9535e58350f36fa8d575ba2ca0efeee,
        0xe6cf7ee8b0c5c1792d5caae8d553026f,
    ),
    // atan(221/256)
    FP492::new(
        0x00000b64fbc2ae18cc1c09de29bd280c,
        0x8a0b6ee7e3ade567914a6dc78e6d2d59,
        0xcf07532f340bf7667216b7231de7d84d,
        0x200035318b2be6e12b2a66a18bd61ac5,
    ),
    // atan(222/256)
    FP492::new(
        0x00000b6e222a8aa9d77901de72a31f5c,
        0x864980ad7a1ec73b3fd15f25b5517622,
        0xe4065e30321a937ad0b6c235a7456c45,
        0x510447851427ff851e99c76e88d40bd4,
    ),
    // atan(223/256)
    FP492::new(
        0x00000b773f88102577ad25dd460cc4e3,
        0xef828f91a49550d67f4979bfbd896e0e,
        0x3d97d67a20c15eebef10583f97104955,
        0x957423d93f8ad2b00266ff13b3aac13c,
    ),
    // atan(224/256)
    FP492::new(
        0x00000b8053e2bc2319e73cb2da55210a,
        0x4443d3d7aecc114c79a80a012155f64c,
        0xae5304655f065cec9818249c49ec7266,
        0x8272ce8c53e214380781c38257e41946,
    ),
    // atan(225/256)
    FP492::new(
        0x00000b895f4214a8cc2b9b2066fb1102,
        0xf77175a4d5a41aa36f1352801763cfa8,
        0x544aadc6367daedafdea6b772d374c75,
        0x0184538d73bdc564de9dfb7e0289b47d,
    ),
    // atan(226/256)
    FP492::new(
        0x00000b9261ada7d73d86522ccb6cd266,
        0xbc46005fde833f28bcf2a279fbde231d,
        0xebfaa85a0e5c8dadbb90059fc20bb5ce,
        0x1e2246a57f0ec1fc7fb020c4ccbf5a92,
    ),
    // atan(227/256)
    FP492::new(
        0x00000b9b5b2d0b96ff9775fd06a892d1,
        0x00e70f01f06d4d5d0f7a02e44e671ac1,
        0xe21d1fe9e8c167f14ee9c36ddd27f0cd,
        0xd73d1929e144a8d2842459f568d99427,
    ),
    // atan(228/256)
    FP492::new(
        0x00000ba44bc7dd470782f654c2cb1094,
        0x2e38623228454d454a3430672fcfd51e,
        0xece2741a41f8d0d634feced6648f637c,
        0x5c425cbe25d8962ea50d2c24f5187ee5,
    ),
    // atan(229/256)
    FP492::new(
        0x00000bad3385c16c6c3dc95898e5210d,
        0xa150b48cca7310a8c7b1ed8ae8b68989,
        0x256d6f8603fbac372d285cda43312f0f,
        0xbf88ee98e80f531a19866a0ce4d29774,
    ),
    // atan(230/256)
    FP492::new(
        0x00000bb6126e636360232a4aa35b9079,
        0x15ae61342deff6f8d6c65e2c7194d05b,
        0x79464902eb9556927dd63892edee986a,
        0xe2a0f3fed44735ae2bca9b58a00b1d83,
    ),
    // atan(231/256)
    FP492::new(
        0x00000bbee889751163ae64c11f821a7a,
        0xe3e62740e9aafa599e7d2877735b920d,
        0x4c84d43eb505394867be0b1618881ff8,
        0xf33bd888ad40d96ee2335ca1d27237ac,
    ),
    // atan(232/256)
    FP492::new(
        0x00000bc7b5deae98af280d4113006e80,
        0xfb29013fab81f5ba4ab8a231796bb551,
        0x4a995010c85fc569845f3d4140c7bd05,
        0x0067c920ba34f5d58da8a994e9987c4c,
    ),
    // atan(233/256)
    FP492::new(
        0x00000bd07a75ce0cd1089e33e6c0afab,
        0xb5d7415d7d6a58a497168be81bf84c71,
        0xfed7904259565f3ec992aafabb0ad11f,
        0x4d92f75a8ca7855ab5af72fc841ff079,
    ),
    // atan(234/256)
    FP492::new(
        0x00000bd9365697287ec62fe419f7dc09,
        0xb179189fb1e26c726f312a593af0f56e,
        0x4bb84384a46715ca4f7fb9f3a1bfe120,
        0x2cd89ce65df39924473f22b300fb3af6,
    ),
    // atan(235/256)
    FP492::new(
        0x00000be1e988d30495b767965a606510,
        0x9b7436bbb76d27fa58b329c2bf459664,
        0xccbbe7e4bb855639ea69ea3c88dfdc33,
        0xc737964640bd9570f1ff01495ad10dba,
    ),
    // atan(236/256)
    FP492::new(
        0x00000bea94144fd049aac1043c5e7552,
        0x82e7d01438341f13d5c3a74fdaef1655,
        0xbf3454130f65d014699b194f33cd5edc,
        0x0ee6314b1de7316036500c26f2caaeb4,
    ),
    // atan(237/256)
    FP492::new(
        0x00000bf33600e08a7ec6fa8ee913e8e5,
        0x0ada66f6bfd26a2606cd81511e852e77,
        0x38e7aec13bb8d7bfd70599801fe64491,
        0x82faa624975d4189c9cdd3b85c8bc949,
    ),
    // atan(238/256)
    FP492::new(
        0x00000bfbcf565cbc4c3fab9501d5a441,
        0xa24b5ce2085df539345586ab1d62364b,
        0x3656a6093845c0de6e85621db31c0f61,
        0x807651a3acefda99fba77905b317a91c,
    ),
    // atan(239/256)
    FP492::new(
        0x00000c04601ca034a55fe19b11d412b6,
        0x30d4d59538baba46a50a5548e902eb9a,
        0x2364286e41e4ea87870f0215fd084488,
        0x0f10e1856411be80b57542234ea74aec,
    ),
    // atan(240/256)
    FP492::new(
        0x00000c0ce85b8ac52664089dd62c46e9,
        0x2fa24d58ee867aef436f637081467a10,
        0xb2d2544ffa53c51a62f57a34ac5cf485,
        0xbc5c23ad03636ac95ccb6dfe86b83990,
    ),
    // atan(241/256)
    FP492::new(
        0x00000c15681b00000294592fce924d24,
        0x25ea8695bf27c703e16c40987beecd15,
        0x750708d6fdbad83833ee6f29f8b05e75,
        0x1787074459366d5ce0fad7d87e16df2b,
    ),
    // atan(242/256)
    FP492::new(
        0x00000c1ddf62e6f711098838827fe361,
        0x448e71812b904e3b003f26653bbab815,
        0x709d175af9aa616023b260458efabead,
        0x7180d4574b346e337a1a4695849102de,
    ),
    // atan(243/256)
    FP492::new(
        0x00000c264e3b29fbf57e7c9ee01f5c17,
        0x13e2f8039a8ec1847c57bfc8ce728ce8,
        0x822765b06891e12dc125dc1a7586fb4e,
        0x62c8d6ed05d326570fc405a07a42e3cd,
    ),
    // atan(244/256)
    FP492::new(
        0x00000c2eb4abb661628b5b373fe45c61,
        0xbb9fae970ec0e0e4baa66a4a42effa97,
        0x24511ca811d6a00e8f7ef16f0247649d,
        0x272f441f3a7c6e8cb14e35c9307919f7,
    ),
    // atan(245/256)
    FP492::new(
        0x00000c3712bc7c3d72a045626eb9fc67,
        0x8fde093a72f30d04b7f6aaa12aec8218,
        0xc6d64c38e91badf6f52680df7a13e115,
        0x9f52ce88a3fcb5ede11895c2a2311629,
    ),
    // atan(246/256)
    FP492::new(
        0x00000c3f68756e2d1510af5ad957f4bc,
        0x61565ce4aa623cab62fb557b7468f306,
        0x1f2f3ca40809d826637a2c86286b19a8,
        0x1f5fb31d1e173dfd54159e2a717f36c5,
    ),
    // atan(247/256)
    FP492::new(
        0x00000c47b5de81188c7b34b8b6951667,
        0xe250297192bb863cb83ea91d85ec0f32,
        0xb4f931ccc9b7ccb63ad190a313650438,
        0xff50d3a4558bd5903eb22c2db39af88d,
    ),
    // atan(248/256)
    FP492::new(
        0x00000c4ffaffabf8fbd548cb43d10bc9,
        0xe02214da621b60039834efe669d994f9,
        0x7d1fab0fd569445c51352a068eb0cf84,
        0x63c2224ed1b160006bc9a098b6ca9d29,
    ),
    // atan(249/256)
    FP492::new(
        0x00000c5837e0e79eff5e099db3524926,
        0x3d1d46837d6da14b2bfbef66d6033b4a,
        0xb95c99ba8c506c61fe6c93498d02083a,
        0xbad85a8c80473b8b30016388e1c1e65f,
    ),
    // atan(250/256)
    FP492::new(
        0x00000c606c8a2e7a4eb7d480edb88665,
        0x164556f74b5da8a15c194442531f379a,
        0x3ff17f2db2abb47a14fa2449d52305ce,
        0x6c6be1893fef759c15a7b149126ae892,
    ),
    // atan(251/256)
    FP492::new(
        0x00000c6899037c626564fe7345576574,
        0xca266dd13c472e3af549fa723e05b142,
        0x2a61b6f575b7c4647ee8f69f86fc8d58,
        0xb13d2cd4c845e57a88b578c2412344dd,
    ),
    // atan(252/256)
    FP492::new(
        0x00000c70bd54ce602ee13e7d54fbd09f,
        0x2be380e9c986eaf9b702aadfde929484,
        0x17fdabd069fabb5848c7ebb9c9bcfde8,
        0x162ac724d1c664175eb9df6c32fc77cd,
    ),
    // atan(253/256)
    FP492::new(
        0x00000c78d9862278b38ee8e55c9c99fe,
        0x946b23c9c7639e8b4d30ad41681b39d7,
        0xe8a06972851975bb954e9b2be502821b,
        0x73f1ee0b01556fc0fe1f44d3fb7296b5,
    ),
    // atan(254/256)
    FP492::new(
        0x00000c80ed9f7778c3ad0ccbcac0791e,
        0xf71d43b7d29bec708214c8402ae01729,
        0x8a4a45c540c8c119a56149a21e260393,
        0xe36826f6ffd2275c0f2951eca4460e95,
    ),
    // atan(255/256)
    FP492::new(
        0x00000c88f9a8ccc19d89d745076dfa90,
        0xd530c167b4c95e7b9333ed9884aae7ea,
        0x0180ba5917ced01e3f55fb0e884ab73c,
        0xeababe0a59182644636cda0d8504a03e,
    ),
];

#[allow(clippy::cast_sign_loss)]
pub(crate) fn approx_atan(x: &FP492) -> FP492 {
    let mut x_abs = *x;
    x_abs.iabs();
    debug_assert!(x_abs < FP492::ONE);
    // Reduce |x| so that |x| = c + y and y < 1/256
    let (q, c, mut y) = x_abs.divmod_1_over_256();
    // atan(|x|) = atan(c) + atan(y/(1+|x|⋅c))
    let mut t = x_abs;
    // |x| < 1 => c < 1 => |x|⋅c < 1
    t *= &c;
    t += &FP492::ONE;
    // 1 < t < 2 and y < 1/256 => y/t < 1/256
    y /= &t;
    let mut y2 = y;
    y2.imul_round(&y);
    let mut atan = COEFFS[0];
    for coeff in &COEFFS[1..N] {
        atan.imul_round(&y2);
        atan += coeff;
    }
    atan.imul_round(&y);
    // Finally add tabulated atan(c)
    atan += &ATANS[q as usize];
    // atan(-x) = -atan(x)
    if x.is_sign_negative() {
        atan = -atan;
    }
    atan
}
