// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{BigFloat, FP509};
use crate::f256;

const N: usize = 33;

const COEFFS: [FP509; N] = [
    // 65: 1 / 65
    FP509::new(
        0x007e07e07e07e07e07e07e07e07e07e0,
        0x7e07e07e07e07e07e07e07e07e07e07e,
        0x07e07e07e07e07e07e07e07e07e07e07,
        0xe07e07e07e07e07e07e07e07e07e07e0,
    ),
    // 63: -1 / 63
    FP509::new(
        0xff7df7df7df7df7df7df7df7df7df7df,
        0x7df7df7df7df7df7df7df7df7df7df7d,
        0xf7df7df7df7df7df7df7df7df7df7df7,
        0xdf7df7df7df7df7df7df7df7df7df7df,
    ),
    // 61: 1 / 61
    FP509::new(
        0x00864b8a7de6d1d60864b8a7de6d1d60,
        0x864b8a7de6d1d60864b8a7de6d1d6086,
        0x4b8a7de6d1d60864b8a7de6d1d60864b,
        0x8a7de6d1d60864b8a7de6d1d60864b8a,
    ),
    // 59: -1 / 59
    FP509::new(
        0xff75270d0456c797dd49c34115b1e5f7,
        0x5270d0456c797dd49c34115b1e5f7527,
        0x0d0456c797dd49c34115b1e5f75270d0,
        0x456c797dd49c34115b1e5f75270d0457,
    ),
    // 57: 1 / 57
    FP509::new(
        0x008fb823ee08fb823ee08fb823ee08fb,
        0x823ee08fb823ee08fb823ee08fb823ee,
        0x08fb823ee08fb823ee08fb823ee08fb8,
        0x23ee08fb823ee08fb823ee08fb823ee1,
    ),
    // 55: -1 / 55
    FP509::new(
        0xff6b0df6b0df6b0df6b0df6b0df6b0df,
        0x6b0df6b0df6b0df6b0df6b0df6b0df6b,
        0x0df6b0df6b0df6b0df6b0df6b0df6b0d,
        0xf6b0df6b0df6b0df6b0df6b0df6b0df7,
    ),
    // 53: 1 / 53
    FP509::new(
        0x009a90e7d95bc609a90e7d95bc609a90,
        0xe7d95bc609a90e7d95bc609a90e7d95b,
        0xc609a90e7d95bc609a90e7d95bc609a9,
        0x0e7d95bc609a90e7d95bc609a90e7d96,
    ),
    // 51: -1 / 51
    FP509::new(
        0xff5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f,
        0x5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f,
        0x5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f,
        0x5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f,
    ),
    // 49: 1 / 49
    FP509::new(
        0x00a72f05397829cbc14e5e0a72f05397,
        0x829cbc14e5e0a72f05397829cbc14e5e,
        0x0a72f05397829cbc14e5e0a72f053978,
        0x29cbc14e5e0a72f05397829cbc14e5e1,
    ),
    // 47: -1 / 47
    FP509::new(
        0xff51b3bea3677d46cefa8d9df51b3bea,
        0x3677d46cefa8d9df51b3bea3677d46ce,
        0xfa8d9df51b3bea3677d46cefa8d9df51,
        0xb3bea3677d46cefa8d9df51b3bea3678,
    ),
    // 45: 1 / 45
    FP509::new(
        0x00b60b60b60b60b60b60b60b60b60b60,
        0xb60b60b60b60b60b60b60b60b60b60b6,
        0x0b60b60b60b60b60b60b60b60b60b60b,
        0x60b60b60b60b60b60b60b60b60b60b61,
    ),
    // 43: -1 / 43
    FP509::new(
        0xff417d05f417d05f417d05f417d05f41,
        0x7d05f417d05f417d05f417d05f417d05,
        0xf417d05f417d05f417d05f417d05f417,
        0xd05f417d05f417d05f417d05f417d05f,
    ),
    // 41: 1 / 41
    FP509::new(
        0x00c7ce0c7ce0c7ce0c7ce0c7ce0c7ce0,
        0xc7ce0c7ce0c7ce0c7ce0c7ce0c7ce0c7,
        0xce0c7ce0c7ce0c7ce0c7ce0c7ce0c7ce,
        0x0c7ce0c7ce0c7ce0c7ce0c7ce0c7ce0c,
    ),
    // 39: -1 / 39
    FP509::new(
        0xff2df2df2df2df2df2df2df2df2df2df,
        0x2df2df2df2df2df2df2df2df2df2df2d,
        0xf2df2df2df2df2df2df2df2df2df2df2,
        0xdf2df2df2df2df2df2df2df2df2df2df,
    ),
    // 37: 1 / 37
    FP509::new(
        0x00dd67c8a60dd67c8a60dd67c8a60dd6,
        0x7c8a60dd67c8a60dd67c8a60dd67c8a6,
        0x0dd67c8a60dd67c8a60dd67c8a60dd67,
        0xc8a60dd67c8a60dd67c8a60dd67c8a61,
    ),
    // 35: -1 / 35
    FP509::new(
        0xff15f15f15f15f15f15f15f15f15f15f,
        0x15f15f15f15f15f15f15f15f15f15f15,
        0xf15f15f15f15f15f15f15f15f15f15f1,
        0x5f15f15f15f15f15f15f15f15f15f15f,
    ),
    // 33: 1 / 33
    FP509::new(
        0x00f83e0f83e0f83e0f83e0f83e0f83e0,
        0xf83e0f83e0f83e0f83e0f83e0f83e0f8,
        0x3e0f83e0f83e0f83e0f83e0f83e0f83e,
        0x0f83e0f83e0f83e0f83e0f83e0f83e10,
    ),
    // 31: -1 / 31
    FP509::new(
        0xfef7bdef7bdef7bdef7bdef7bdef7bde,
        0xf7bdef7bdef7bdef7bdef7bdef7bdef7,
        0xbdef7bdef7bdef7bdef7bdef7bdef7bd,
        0xef7bdef7bdef7bdef7bdef7bdef7bdef,
    ),
    // 29: 1 / 29
    FP509::new(
        0x011a7b9611a7b9611a7b9611a7b9611a,
        0x7b9611a7b9611a7b9611a7b9611a7b96,
        0x11a7b9611a7b9611a7b9611a7b9611a7,
        0xb9611a7b9611a7b9611a7b9611a7b961,
    ),
    // 27: -1 / 27
    FP509::new(
        0xfed097b425ed097b425ed097b425ed09,
        0x7b425ed097b425ed097b425ed097b425,
        0xed097b425ed097b425ed097b425ed097,
        0xb425ed097b425ed097b425ed097b425f,
    ),
    // 25: 1 / 25
    FP509::new(
        0x0147ae147ae147ae147ae147ae147ae1,
        0x47ae147ae147ae147ae147ae147ae147,
        0xae147ae147ae147ae147ae147ae147ae,
        0x147ae147ae147ae147ae147ae147ae14,
    ),
    // 23: -1 / 23
    FP509::new(
        0xfe9bd37a6f4de9bd37a6f4de9bd37a6f,
        0x4de9bd37a6f4de9bd37a6f4de9bd37a6,
        0xf4de9bd37a6f4de9bd37a6f4de9bd37a,
        0x6f4de9bd37a6f4de9bd37a6f4de9bd38,
    ),
    // 21: 1 / 21
    FP509::new(
        0x01861861861861861861861861861861,
        0x86186186186186186186186186186186,
        0x18618618618618618618618618618618,
        0x61861861861861861861861861861862,
    ),
    // 19: -1 / 19
    FP509::new(
        0xfe50d79435e50d79435e50d79435e50d,
        0x79435e50d79435e50d79435e50d79435,
        0xe50d79435e50d79435e50d79435e50d7,
        0x9435e50d79435e50d79435e50d79435e,
    ),
    // 17: 1 / 17
    FP509::new(
        0x01e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1,
        0xe1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1,
        0xe1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1,
        0xe1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e2,
    ),
    // 15: -1 / 15
    FP509::new(
        0xfddddddddddddddddddddddddddddddd,
        0xdddddddddddddddddddddddddddddddd,
        0xdddddddddddddddddddddddddddddddd,
        0xddddddddddddddddddddddddddddddde,
    ),
    // 13: 1 / 13
    FP509::new(
        0x02762762762762762762762762762762,
        0x76276276276276276276276276276276,
        0x27627627627627627627627627627627,
        0x62762762762762762762762762762762,
    ),
    // 11: -1 / 11
    FP509::new(
        0xfd1745d1745d1745d1745d1745d1745d,
        0x1745d1745d1745d1745d1745d1745d17,
        0x45d1745d1745d1745d1745d1745d1745,
        0xd1745d1745d1745d1745d1745d1745d1,
    ),
    // 9: 1 / 9
    FP509::new(
        0x038e38e38e38e38e38e38e38e38e38e3,
        0x8e38e38e38e38e38e38e38e38e38e38e,
        0x38e38e38e38e38e38e38e38e38e38e38,
        0xe38e38e38e38e38e38e38e38e38e38e4,
    ),
    // 7: -1 / 7
    FP509::new(
        0xfb6db6db6db6db6db6db6db6db6db6db,
        0x6db6db6db6db6db6db6db6db6db6db6d,
        0xb6db6db6db6db6db6db6db6db6db6db6,
        0xdb6db6db6db6db6db6db6db6db6db6db,
    ),
    // 5: 1 / 5
    FP509::new(
        0x06666666666666666666666666666666,
        0x66666666666666666666666666666666,
        0x66666666666666666666666666666666,
        0x66666666666666666666666666666666,
    ),
    // 3: -1 / 3
    FP509::new(
        0xf5555555555555555555555555555555,
        0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
    ),
    // 1: 1
    FP509::new(
        0x20000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
];

const ATANS: [FP509; 256] = [
    // atan(0/256)
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
    // atan(1/256)
    FP509::new(
        0x001ffff5555bbbb72976255f6d6da9ef,
        0xc2cf83175d73792af1244915a4d057de,
        0xcdad10d01ec4b9bf7ac48a1acc24deb5,
        0x3b13d77c8cf7c75c28e34a6b9001ead8,
    ),
    // atan(2/256)
    FP509::new(
        0x003fffaaab77752e5a0188d47eef982b,
        0xd1da15a805c68bc006a0f455d9fbb394,
        0x24223a10251791eb2df396f8b58949ca,
        0x598ef21db2e2fc3caae1e53f1af35c55,
    ),
    // atan(3/256)
    FP509::new(
        0x005ffee006130c268dafe8af7dcb4b6e,
        0x2de7685c837dbbb1c29b61d0f589a3f5,
        0xbb8957091d7d47c8f3cc116b9225297d,
        0xa8d5ed77fc830c604366e863ac3d1ea3,
    ),
    // atan(4/256)
    FP509::new(
        0x007ffd556eedca6addf3c62b200afbb0,
        0x241077b2e086f77a3034afc0193ab0e5,
        0xd46d43bd30b1a3caed1a1326c50dd374,
        0x97957c02cf2f5a9290b4371f69bf3843,
    ),
    // atan(5/256)
    FP509::new(
        0x009ffacaf8c537fdbd82a792609a7fa4,
        0xffa60d604a401d620ac66b4b146264f0,
        0x871a26c898bbc6322e69a45422cb4d4b,
        0x46527862240d40f0ad6a551334b41972,
    ),
    // atan(6/256)
    FP509::new(
        0x00bff700c252e1ad79cec07eb956a4a3,
        0x71ba9f702d55db5458939a42a9586122,
        0x6dc423ca71344499e6d4f5ad3817df0e,
        0x15391f55797404339074e6c73411dbe9,
    ),
    // atan(7/256)
    FP509::new(
        0x00dff1b6f948b88e1e41388a9eff195f,
        0x9968f34b8c2e349753555fdfe05ffb67,
        0x8c64a6810a9310fb58c9979382ebd09f,
        0x90ea1e7454a039411af552e6b88d17f0,
    ),
    // atan(8/256)
    FP509::new(
        0x00ffeaaddd4bb12542779d776dda8c62,
        0x13806d0294c0db881647017db44d6106,
        0x0765523eb12fcbf3a1e505febef487c5,
        0xa2790d24f84ed5f8826b192b8de6c520,
    ),
    // atan(9/256)
    FP509::new(
        0x011fe1a5c2ec4976218247a3f132d145,
        0x549fee8bc378d0570703a6b326f3dc97,
        0xda37802acebf2d048e9f9241b53b9a4d,
        0xc46f93b5e532e9cd7f8fc39190f39145,
    ),
    // atan(10/256)
    FP509::new(
        0x013fd65f169c9d917230a716461b4e5e,
        0xc309d9936441b71180e53273affa5b40,
        0x4141d05d2fadf5832652e3d1745802be,
        0x95de8934dddcd71a532783d0cb3a42c3,
    ),
    // atan(11/256)
    FP509::new(
        0x015fc89a5fa3b2d4aedcefd39fe62e29,
        0x078c9aab6a4b4e5556d7b04ecf8518d4,
        0x536040b0ebe5c2479d89601b3f95ea7e,
        0xc15be682f9323640b126c5c7f6058b28,
    ),
    // atan(12/256)
    FP509::new(
        0x017fb818430da29f9e441c21ac3beadd,
        0x8e3bbbac9f00fe4114ee624042fbcf2e,
        0xd4b312c4c87377e3dda112970ce2e02f,
        0xb3629dad1aeab66dc4af61c1c2c58f21,
    ),
    // atan(13/256)
    FP509::new(
        0x019fa49986984df4c8a41ebc2af072b7,
        0x4b08e041adfbb67db6538cb69cdb01e7,
        0x51fc00cb23ce0fffc04e919fa3e1a30f,
        0x05ae1b595bf7656820d152de3c6ed7a0,
    ),
    // atan(14/256)
    FP509::new(
        0x01bf8ddf139c4439d8072d3560161d75,
        0x934a3e53d2b497ca01a367cb951328c6,
        0xb6bff8ba0af5f178d7c000a73a89e59c,
        0xe5df21ba0e947fadddeb58f810c21968,
    ),
    // atan(15/256)
    FP509::new(
        0x01df73a9f9f1881f6d7251df7a1a5f9f,
        0xedeb3a33d81fb51cf4b05a75d40f78be,
        0xc0190ca069434dd6d0910fa78a899c21,
        0x5a33bc55331b9fa9e793a9a01a2ecdaa,
    ),
    // atan(16/256)
    FP509::new(
        0x01ff55bb72cfde9c6d964f25b81c5c1a,
        0xa26338259eb3a965b870f53f00789715,
        0xda04920128e7d36fa927fc524d58e900,
        0x76280967cc4b4981484a43bd29de550b,
    ),
    // atan(17/256)
    FP509::new(
        0x021f33d4e3aa54dedf960113c9db1233,
        0xb19f699da91f47d80d2038a328ef680c,
        0x38d737f22cca24f88e9fb7d671e38ee6,
        0xebb72442aac6c217209bffee06226ad5,
    ),
    // atan(18/256)
    FP509::new(
        0x023f0db7e105ab1bda88e233441fb61f,
        0xca53aab183c2b48e9c8700a159cca170,
        0xbe7f78ed0ffbb0bbf28fbf8a409af810,
        0xccab99fcfdfe064d60a16d86339c3497,
    ),
    // atan(19/256)
    FP509::new(
        0x025ee326314953404ff45de6e6007cb9,
        0x5444a33d8a316533401cdf74780e25ea,
        0x9c725f444b9efab41cc142c2d6105e81,
        0x5f0658125e4f3110485e8416f5408c6b,
    ),
    // atan(20/256)
    FP509::new(
        0x027eb3e1cf8ab3ad6267142efc239b37,
        0xd93d661f697e6f219b8d6c2fa9303110,
        0x40f196120da7867686d5e2d87dc2fde0,
        0x51dad4de792fadb7f4b467bcadece1a4,
    ),
    // atan(21/256)
    FP509::new(
        0x029e7facee525f640234d89925e765e3,
        0x7c9a8bd53d42e909a7dde62bfc3cd243,
        0x156c70cca9e235a936edbf694cc7c505,
        0x21626235ddc5f2aae637f8cf2d3a1364,
    ),
    // atan(22/256)
    FP509::new(
        0x02be4649fa5af64c546d220098c6fd59,
        0xce6bf418a3815f5830a217c1d5a2116a,
        0x86c1c2187d2e683d18147130fe2f45e4,
        0x2777fc5a5b32fd2786ead85f28b47688,
    ),
    // atan(23/256)
    FP509::new(
        0x02de077b9d49619d81fe35da0bd06bfb,
        0x29245e6af63bae8b96299a9c05a11d17,
        0x1edfd551c5d4805744c44c9c5c27572e,
        0x977e0f9ba157e72f1262994c9fc8aab2,
    ),
    // atan(24/256)
    FP509::new(
        0x02fdc304c05e21d1824d59f9e133e0d9,
        0xd9f82a11e60ab8370caccfc0b1867c01,
        0x68a8e99054926648d8e00d8d79f89dda,
        0x160a6c5ba28f995a1ec396a41d3dec75,
    ),
    // atan(25/256)
    FP509::new(
        0x031d78a88f1f6505cca2032b4932ea1b,
        0xd8f16ec470721363d7b0de76c478991a,
        0x03729fcb7abbf9d7671b3b65cdf0d3cf,
        0x44acd58d3a8f98b0469e25696952a45f,
    ),
    // atan(26/256)
    FP509::new(
        0x033d282a79fb9e2d19970eb4e23f02a3,
        0xd3f521944e4ffd8951923cdc649e9d07,
        0x383b17ed4fd65eb520eeabebdb875c11,
        0xb773fcf2fb72ab9b256ba58eeed6db27,
    ),
    // atan(27/256)
    FP509::new(
        0x035cd14e38e4570700539a442e76dcb7,
        0xf77fc9153b11d742258e1747cf3035d4,
        0x3dd104e2a01732379534a15b0b2f6faf,
        0x1f433e285a5962908da569648d7f8d2e,
    ),
    // atan(28/256)
    FP509::new(
        0x037c73d7cde0f86fbdc794d02cc39767,
        0xba435f943a46884fb6361c17ad0cc9ee,
        0x92366de23c21d0bdd7e580e5d352e085,
        0x79caaa8141efa900cf6ac706613266be,
    ),
    // atan(29/256)
    FP509::new(
        0x039c0f8b879946444e3990964629a518,
        0x32cf73bb04f8fca0b6da8a1f4b9689a7,
        0xc29967b511a5dad59f8a8d81cee0f704,
        0xa992983438159518b3c7eb6da85890c0,
    ),
    // atan(30/256)
    FP509::new(
        0x03bba42e03d74dcca77fe6bb7f2bf56b,
        0x322162d35e686b6516d1de92f378190e,
        0x20517f458cfae9d3c8dc09d989acbd12,
        0xd5a537d911aaa12810b3ae06e6af94e3,
    ),
    // atan(31/256)
    FP509::new(
        0x03db31843200875ec5a67a6d384ed529,
        0x1d3c6db08cacf3083faebd41ced81721,
        0x4df67c3957133232b0612c7eb76cea16,
        0xa5c6f4bfd945aae12f2a77ff2fc4bda9,
    ),
    // atan(32/256)
    FP509::new(
        0x03fab7535585edb8cb225e627cfa223b,
        0xde2ce55f65766b648bb24d5df7edb05d,
        0xa35340200807670994ffe9754788d35c,
        0x7575cf2627ca6beff4a9b3622afe11be,
    ),
    // atan(33/256)
    FP509::new(
        0x041a3561084acf661d2d7cfe6402ed6e,
        0x9f971f7dacb5fdb6963787347f589996,
        0x3f9c01c8c586a0266f27cedd37dd2c77,
        0xcfbedb1d229df90eaaa8fda48d4f5c00,
    ),
    // atan(34/256)
    FP509::new(
        0x0439ab733d0220628e5ce44672f2c72d,
        0x09b0f5ef1f01228510df0eb0c095066c,
        0xbd4926ac3cc55883c42ff19754af0e18,
        0x7b686e5f2528e93250242ec511cec77f,
    ),
    // atan(35/256)
    FP509::new(
        0x0459195041821416bcee8ae7ea91abc6,
        0x0c5253b6eeff1270ff27618b78d1f4d3,
        0x7c44c6ea6fb7e090909a0a9a053d3e52,
        0xf65be7951682285106c5e00a4c77ab77,
    ),
    // atan(36/256)
    FP509::new(
        0x04787ebec10dc9b92de9bac94c1d057c,
        0xcf150ad614fe006473005883817d208d,
        0x1b0d02bcf303e81b5ed0ec4b199eab0d,
        0xcf849efb6d4e04e20584bace1a4c41ea,
    ),
    // atan(37/256)
    FP509::new(
        0x0497db85c694d717c6d7bde1a31079c3,
        0x95ae342442076e7200532e183333c011,
        0x2bbe5a7c16b073f8e7b900cf883bd0d6,
        0x219ff0022fb037aecf5b46b8a4532042,
    ),
    // atan(38/256)
    FP509::new(
        0x04b72f6cbee87fcc794382ab0a9a29f9,
        0x5681fa4e35878c271c43dfe58123fe42,
        0x5226b773212741c6be47e5a5e825de93,
        0x105a8d2bef1d78feea6a59e3901d49af,
    ),
    // atan(39/256)
    FP509::new(
        0x04d67a3b7ae668e5a028b1b9fb39c57a,
        0x616b7a48b6aefd840790ad0a3fb8870f,
        0xb9accfa30fd88536dc275a209660fbe0,
        0x98c2d28b83812db358634b0a9288b143,
    ),
    // atan(40/256)
    FP509::new(
        0x04f5bbba31989b161a3b0ce9281b07f4,
        0x2bba97a7c5d6e946e265e0525a14780b,
        0x6dac1d47778d549f9bc936b61515b947,
        0xf769014838dfb58e549a45ced7c5a276,
    ),
    // atan(41/256)
    FP509::new(
        0x0514f3b1824aa791f653b3a5a78b70b1,
        0xfb2a0105322cb94e4f21a978877a0338,
        0x826765661aadade87e4844eb6cbd90a3,
        0x6c5e55152ca6c2be85fd3bdfdc8ee9fc,
    ),
    // atan(42/256)
    FP509::new(
        0x053421ea7693c5d1f323f1adf157983e,
        0xf0bf041414c052e9ee3cf68fa7f96a9a,
        0x02d06cb4a403b9f499c1eb6baf84f7b9,
        0x0c61d32cfa592cd514928ee556494fc1,
    ),
    // atan(43/256)
    FP509::new(
        0x0553462e8455c2918282f28840737595,
        0x2a5ed187a30c0ffd7f33147df89ef2c0,
        0xef1710953fdff502ac1693ec2dd060d5,
        0x25772b25b30e38056a3ef3acce5420af,
    ),
    // atan(44/256)
    FP509::new(
        0x057260478fb09a77d5aa69ff78616f4c,
        0xa169f36ea8d835a660fb1b299e93dea6,
        0x04a298e2bd6cf91b949fadb498739b3e,
        0x37f01bbb60202cf161a8cc3b6e044fa7,
    ),
    // atan(45/256)
    FP509::new(
        0x05916fffecea9cfc159293107a698de2,
        0x231fbbea98da5012c889d51a56ecdb85,
        0x9b64b253476096409761e774623158d1,
        0x32a7e3d79ac5b74efd65ce3a6e90b5cf,
    ),
    // atan(46/256)
    FP509::new(
        0x05b07522624cf636b55ac4fe1deca934,
        0x169ff7190f210787c385224fe431569b,
        0x2f1df0702f1d943c7744d06343d30a08,
        0x5729ffda8cfa4fd423fec4e6976fb63c,
    ),
    // atan(47/256)
    FP509::new(
        0x05cf6f7a29f480752978495accd1abe0,
        0xf16c44852efed81434984332c0125a69,
        0xa59563a482fd61a45ffee28f8c8868e0,
        0xe36b53560e016caefb5055fd614ac3ed,
    ),
    // atan(48/256)
    FP509::new(
        0x05ee5ed2f396c089a3d85a7c40e4e3a4,
        0x3c9d6806b41c51595782d35278f3eb96,
        0x8cb88230742628e8145c1c6fa7eb805e,
        0x7c6a8c08211abbf00550368b6a9f18a8,
    ),
    // atan(49/256)
    FP509::new(
        0x060d42f8e63af1f52b7666a31b79841f,
        0x707552b2415bccda9db8b32471174deb,
        0xff242f844bd67a5bf147102cb1598ff2,
        0xc91e4d8837e12abf657ff799dfdfa861,
    ),
    // atan(50/256)
    FP509::new(
        0x062c1bb8a1e70a2ee732cfcf73b0be58,
        0xb4f2fb2186728017bdd73a7ef46da1c1,
        0x937455229f1b4a9d0a6e8ef4a14f406f,
        0x40c8b0c7c3b147149a043821a877e683,
    ),
    // atan(51/256)
    FP509::new(
        0x064ae8df41409c6f2c212f00b89c3d8e,
        0x37fa7d1d221df6596f082ed49ac42691,
        0xdbecd6dea83dee8d66e245e1ae695b58,
        0x6445ed603b1b9d38bcbec0927913d6c2,
    ),
    // atan(52/256)
    FP509::new(
        0x0669aa3a5b2189873d8079ed0d237759,
        0xd55f893260de13b0df82f2db444499e1,
        0x47ca0c034c98a036909eddbfc3706ad7,
        0x54b0305e7639a446f7ec8dff0bba33b7,
    ),
    // atan(53/256)
    FP509::new(
        0x06885f980420696f210759f50c8c79e7,
        0x71b14f5af0cc9b3b09c1f36112524e0e,
        0x29b27dc9732796b2311af4d92697b277,
        0xa3d8045bb9fae34b68cd5f5eb20f3727,
    ),
    // atan(54/256)
    FP509::new(
        0x06a708c6d00c9e50e8bbe89cca8550eb,
        0x0d213828a278f1ccb05d91a12eb7a7d4,
        0x53e49e5d16281b3dc4959a65295f366f,
        0x5dce86cb0ddc778e890585c117127c3a,
    ),
    // atan(55/256)
    FP509::new(
        0x06c5a595d35e02f3d2a381f52cc67e91,
        0x593e9cd58301db5f0770f03274370294,
        0x13257c1e0fb00a5b523b3d54bb43a020,
        0x49c3c3f19b4f6fe9b53c45e007957b15,
    ),
    // atan(56/256)
    FP509::new(
        0x06e435d4a498288117b10d2e0e5aa978,
        0x1432289469d25892aab10378cbfc2147,
        0x6218d57a026ee03aa54fac56b7759cbb,
        0x037ef118f37c6c26babe5e45c5b2a7df,
    ),
    // atan(57/256)
    FP509::new(
        0x0702b9535da119afb3240aa0d924277d,
        0xb25e6178a60451a7217d24f8effe0759,
        0x9ef04afce804746a9e56db678117f10d,
        0x9e655aeab454791dce379f71e16b96a1,
    ),
    // atan(58/256)
    FP509::new(
        0x07212fe29d0b9b7352117cbe8aecdaa4,
        0x32d3ef3858475eae6c60501baa8d6fc1,
        0x00ef5217129f114526b08b56ddf7da97,
        0xe0b8fa864966eaf71117470906932bf4,
    ),
    // atan(59/256)
    FP509::new(
        0x073f99538754e5547b9a3f71eafb58ec,
        0xd9847e7623f08f898b913dd6468b3489,
        0x7c92fc4c5f69380b2c1045f23c1bb9a0,
        0x612f0cbbac6e295695b18659e121fac8,
    ),
    // atan(60/256)
    FP509::new(
        0x075df577c815cd9c648d1534597debc7,
        0x3d89676150197f802f36d0cfd144ed80,
        0xc3c9c97a2ac1ec40b7187cebfaf94d97,
        0x9894ea45708778efdf2ddfdeb39498ba,
    ),
    // atan(61/256)
    FP509::new(
        0x077c44219327677d4fce9588170feca1,
        0x238d157d316f4e7863a19773ee220b0d,
        0x5d880c8937fa12642431403b15d5383f,
        0xace749f2993a39cfb5b91d71773149d6,
    ),
    // atan(62/256)
    FP509::new(
        0x079a8523a5bb135866b22029f7650340,
        0xe677edb5e97ff239e2722f3680223a3f,
        0x754c862931c04ef07b7ba6fdb8876d85,
        0x3bcb87d05432c7580b21ee93a9887acb,
    ),
    // atan(63/256)
    FP509::new(
        0x07b8b851476603332c47b9dca2b7a847,
        0x923afb59b67ad958c492f5ea65363c1e,
        0x61333f29d6514135b54ba5c98fd66df9,
        0x8e02636f64c5d97bfee9ee2a446328c4,
    ),
    // atan(64/256)
    FP509::new(
        0x07d6dd7e4b203758ab6e3cf7afbd10bf,
        0x2d53fd481c459c1b5bd1d3b3e4a24d3b,
        0x2c95c928b3472bb2982df462dd2c18d1,
        0xf1e15e113838f27cd07a0e1d581ccc1c,
    ),
    // atan(65/256)
    FP509::new(
        0x07f4f47f1036f904dc51bce8717a9e56,
        0xe689efd436bb31a2dbe9ee3335868fa7,
        0x1da63901da916d7ef90b613ca46709ec,
        0x770668e16067465f06f1f2c21beaaa03,
    ),
    // atan(66/256)
    FP509::new(
        0x0812fd288332dad31a412f9a0b70b4a5,
        0xe253a023b59c459de7d56425c774ec30,
        0x5b4bfb16e97a603dcaa45b92bba4de2c,
        0xf03a4bb246df522cee032ef6c45bd974,
    ),
    // atan(67/256)
    FP509::new(
        0x0830f7501eb1487a8227c93aaf3fc2d4,
        0x1fe3fbb5e3fa9a7180cbfa31e7fce131,
        0x4e2a88e54c47568fdb1b0e4f614fab98,
        0x8e42e0889152568875744d5a451880a3,
    ),
    // atan(68/256)
    FP509::new(
        0x084ee2cbec31b12c5c8e721970cabd3a,
        0x2ae250aa0b6fd05b0e848619404b3045,
        0x05c168a6653fb9ebd71cd6c84466cad5,
        0x87ddc8ae607fa3d85f4180041f65e65c,
    ),
    // atan(69/256)
    FP509::new(
        0x086cbf7284d659a8f862f7ab47d5c176,
        0xf35e7fa8140bc5c1afc17c102ef94d6e,
        0xfd83e29136362d644e877baee35b85f0,
        0x8a0c856b19e0d7d6935a5ee96b2cea33,
    ),
    // atan(70/256)
    FP509::new(
        0x088a8d1b1218e4d646e46df108c27903,
        0x52f3b34317414487b0eda9fd41abd105,
        0xea966337f81d915fecb6a958b152ff27,
        0x24cf597388da7772f467a9d006c6d609,
    ),
    // atan(71/256)
    FP509::new(
        0x08a84b9d4e72a350d28b00c6b9b0c211,
        0x697e2e4da005b7dc67f389f2a916ec51,
        0x679afd366e74ccf3b26030d6833889e1,
        0xb73c233c1be7dd7cdf38ff83e4a5378a,
    ),
    // atan(72/256)
    FP509::new(
        0x08c5fad185f8bc130ca4748b1bf88298,
        0xd0fd2e29bc1a4fbcfca4ebbad508a04e,
        0x8515edb2b719044dbdbe6635b5c51868,
        0x0ad758623cece69e93f9801507754100,
    ),
    // atan(73/256)
    FP509::new(
        0x08e39a9096ec41e826538734d0d24d81,
        0xa2262e81deb73bdbd1b137de85bd2bb5,
        0xae964a45db5f3efca453df2d2e5bb64d,
        0x20fdc650656b9032ec14f222ca18d5f7,
    ),
    // atan(74/256)
    FP509::new(
        0x09012ab3f23e4aee7cdcd70bd6f19982,
        0x3e55be15d40c0c9422be410328a28df3,
        0x969d0257d641298c5c19db2b14a11f57,
        0xdcd5a89f5c01227d36947a4129bad2d5,
    ),
    // atan(75/256)
    FP509::new(
        0x091eab159c0820f1d67474efffbbd7d3,
        0x341a1219e8ac96559530bf4f9248958f,
        0xb4fd16b4c284f984bcfb259927308085,
        0x510e3df5dc42d75d2054f4021e575cbd,
    ),
    // atan(76/256)
    FP509::new(
        0x093c1b902bf7a2df1064592406fe1447,
        0xa447c219f20d6322c7af1667473086f3,
        0xeeeaa1fe7a9c55ed8a222ee93299cce5,
        0xe7351b58625ee1e0a9910e049e5ba75e,
    ),
    // atan(77/256)
    FP509::new(
        0x09597bfecdaff101423cfc1c2d442915,
        0x5e99e34a55bd89de17823b099f6e97a7,
        0xa7b69779908805a5a3c5583abe24da7c,
        0x5678aeee260b066365341bfc407a1147,
    ),
    // atan(78/256)
    FP509::new(
        0x0976cc3d411e7f1b91258ea012c9a1bc,
        0x69f3c7de4985738bd5a94c399d23de5d,
        0x6c54d436daf7e5ecfe22accd26002f78,
        0x3fb90aef00921514e1953643eafb411a,
    ),
    // atan(79/256)
    FP509::new(
        0x09940c27dac4a8cad41eacd8c1a1ba43,
        0x143ef828bb3488d058ea1104ac133e02,
        0x787a44236d901a897bd3a37797ea7e83,
        0x527d51b5331a2950af43f3c84ee00a95,
    ),
    // atan(80/256)
    FP509::new(
        0x09b13b9b83f5e5e69c5abb498d27af32,
        0x808a48f850b2f55fff5a5900c1304c7a,
        0xed338d324a9ddf7e644067ef468c8745,
        0x904ce89259d4d52dbbb1c058d2f69390,
    ),
    // atan(81/256)
    FP509::new(
        0x09ce5a75bb0abdda164b4bbd71d91611,
        0x4354d6381d1590d56f054441794e1db7,
        0x2908cee07b30a0a778c7b029b138048f,
        0xe481031470ee091bc5e23f13eed47143,
    ),
    // atan(82/256)
    FP509::new(
        0x09eb689493889a226f9483992b13d19c,
        0x331ccbdbe5669c6d5075511b28ac2bc5,
        0x02249f688591c9e231e81451eb1459dd,
        0xcb11bd7126a03780b0f204e3c05a2e79,
    ),
    // atan(83/256)
    FP509::new(
        0x0a0865d6b63e9949e6e8a3d895db7ace,
        0x39ad2e0afc5a180faab961b1e2f84f7c,
        0xefe21b177901dbd5e98237e78925ccb5,
        0xe4f77b0dbddfe37ccaeec71413cd5d41,
    ),
    // atan(84/256)
    FP509::new(
        0x0a25521b615784d454378754988b8d9e,
        0x34e8278a4345d860bda441337ceb03b6,
        0x3db3d83b44d9ef5519e20445e9e76b7b,
        0xcd87cec6674920fca8a5ee6e1451c106,
    ),
    // atan(85/256)
    FP509::new(
        0x0a422d4268610da3c514eac78d4b28c8,
        0xb9629add4c41ef4718fb34e2e284e0e7,
        0xc20bdd91da60234419eb23a259ba31b3,
        0xef2c02a0fa52260a07bdfd8e2de23340,
    ),
    // atan(86/256)
    FP509::new(
        0x0a5ef72c34487361b23883dd58a93abf,
        0x402914db410cc323e2f259b477e2986e,
        0xc437d0883be7e4b83ccfc555949ff3c9,
        0x43de38786d5f722446aaf767ebc084cb,
    ),
    // atan(87/256)
    FP509::new(
        0x0a7bafb9c34cbc7352f18400777bcbab,
        0xab0f899e86112a03bca733e3cb72d387,
        0x644349c59153a51b00993e6c1d4e36b4,
        0x97cf209aec540ecc8a51ea31a7a004d9,
    ),
    // atan(88/256)
    FP509::new(
        0x0a9856cca8e6a4eda99b7f77bf7d9e8c,
        0x1212c39d31ef4d7d3a25d29d4c9f1e2b,
        0x6e0fba97ff2614377243ba936bcc3929,
        0x1b556f8f6c32ee524bfe5c4121cdb349,
    ),
    // atan(89/256)
    FP509::new(
        0x0ab4ec470da66be0139e18ec301d8f4c,
        0xabb77105a6898b47c614bcb325aa47d1,
        0x147b669b6d70d539b425a046820d325f,
        0xf0aa65d370f566085b07fa7e04bcd837,
    ),
    // atan(90/256)
    FP509::new(
        0x0ad1700baf07a72276f51f05c8ae83bd,
        0x69db46278dba956160dd5d9b6ea5afc6,
        0xf38d7841431384e028e993fe2a170139,
        0xff83a77b43cfc128036f166d8dc04bb4,
    ),
    // atan(91/256)
    FP509::new(
        0x0aede1fddf3b469ea0ec1b76f7d9c4bf,
        0x659738921ceb2bb025a0e0126a1ec698,
        0x95ded5fd96fc64a7a54f3b37ebb65815,
        0x58c61741f6b5b238cdc6991906086a3e,
    ),
    // atan(92/256)
    FP509::new(
        0x0b0a420184e7f0cb1b51d51dc200a0fc,
        0x2b4085cde97ad6d702fddf1ed0a00a4d,
        0x1af82060036f8bbdb95cb0f363281dc1,
        0xea3680386b1112afc5958fdc01ee67e1,
    ),
    // atan(93/256)
    FP509::new(
        0x0b268ffb1ae0e2c0b7f8307e0484613e,
        0xb0915f2c6f9b78f6b3eaaa3a1602d19e,
        0x47ad7b382dd265cc2422430615f4d04c,
        0xdb0a26036d734b708864f842f68951a5,
    ),
    // atan(94/256)
    FP509::new(
        0x0b42cbcfafd37efb6a1facb94958f693,
        0x56b0ae965b2b4a2b68071b5506db7369,
        0xf6c053b8a98ac27c7c8b0e446b075236,
        0xff5eaf227d449e3f5dc0396d8df0f5d7,
    ),
    // atan(95/256)
    FP509::new(
        0x0b5ef564e5ebb672d1d4e0c90bec0c5f,
        0x4aed2895943d8ed1eee220edbb8a934d,
        0x625c45d5e4a37d589bfae20d71854af4,
        0x091df4e376e3e2f16850f4a19b21799f,
    ),
    // atan(96/256)
    FP509::new(
        0x0b7b0ca0f26f784738aa32122dcfe448,
        0x33d843977ae5455cc4733e5ca334908e,
        0x064cc582039fdc0d6dccce4a800cf73d,
        0x28616ee077afa1a60487d5aad34939d6,
    ),
    // atan(97/256)
    FP509::new(
        0x0b97116a9d5154c4c6d9630c1b0862a3,
        0x07bea22db7f2f62aeb2dbf3cb97546d6,
        0x3271f327eaee87e7a881992ac6c64370,
        0xf8654ca30e0378b8bcd997e0e29f002b,
    ),
    // atan(98/256)
    FP509::new(
        0x0bb303a940ba80f88bba1b8ce27fff42,
        0x7c7b1530d314ccd22bf15a09d156ea18,
        0xb5d852315e86f05b9b5bff5072cf5e47,
        0x6e4cecfc48a77909ac4a68d8853b867a,
    ),
    // atan(99/256)
    FP509::new(
        0x0bcee344c88c6881b7413a0ef606ccf0,
        0x1c55b0788e97c1542f69a3f01d2b71da,
        0x8b4191807f963fc258f4c7cb40cfe394,
        0xa3fd752ae59e10526a14002016f7fb26,
    ),
    // atan(100/256)
    FP509::new(
        0x0beab025b1d9fbad3910b85649341102,
        0x625c13c1d5e5df73f963f7de43ce9d8b,
        0x240a53b5579a414c88facbbf9128f199,
        0x77464c22e69fa5992e298e9a48eb70d9,
    ),
    // atan(101/256)
    FP509::new(
        0x0c066a350a58e8430c0cd2f60bcd98bd,
        0x8c31851f06e5f28492136f8ad2a8c0ae,
        0xa997f77f9936300529895397306c5ab4,
        0xb3aa65b59d594b39a718b27068ec9ae6,
    ),
    // atan(102/256)
    FP509::new(
        0x0c22115c6fcaebbaed405336836b8747,
        0x702eb403911a9b82655434b3413c2355,
        0x1ffcfbb8130820f5d8ddb4a89bf6baab,
        0xeed491b53c7c95a46d4faddcf08b27b7,
    ),
    // atan(103/256)
    FP509::new(
        0x0c3da5860f5f6dd2460aba0b71e1728c,
        0xbab00ae9eee70dfb4ce66bfadfb85bf1,
        0xedc96b697c9b1b26582c09f86333f2e5,
        0xd4ec95759332e510d8d641e73a03ae9b,
    ),
    // atan(104/256)
    FP509::new(
        0x0c59269ca50d92b6da1746e91f50a28d,
        0xdc4b7c9e50d3aa1f6827bb94bb093487,
        0xdc7043ba87431ec654691de993e24b63,
        0xb7c403be536b8c7cbd73c429967d41ce,
    ),
    // atan(105/256)
    FP509::new(
        0x0c74948b7ae6f429814fc257f5f5e868,
        0x7dc1bacd3d66c0d03a59bd07092403fd,
        0x00b0e7ef4d6d6b5272ad30af049b1642,
        0xdaf42bf1df98a03f7184ddeb3a76b5ca,
    ),
    // atan(106/256)
    FP509::new(
        0x0c8fef3e686331220e83100a96a6d7d8,
        0xc6fa642491725beb66f4f40af817add6,
        0xf3eadcf4b30bfd79a47a90f82d00dd88,
        0xeb16cbc08ee80dc3a4f981d175f033b7,
    ),
    // atan(107/256)
    FP509::new(
        0x0cab36a1d1a48399b01537e0af2b5f15,
        0x32e7c293cd15fc9fcf8623447bb614b8,
        0xe8b2be10e6d00c55770b37d785df17a9,
        0x7306561098628fe7afc1751cb894af92,
    ),
    // atan(108/256)
    FP509::new(
        0x0cc66aa2a6b58c33cd9311fa14ed9b7c,
        0x3927a0054882196141323666edbd4549,
        0x02066c4fa234b4ce55ce0e83b7b53d06,
        0x70c156f27f2827d3a5aa0763b4f4fda0,
    ),
    // atan(109/256)
    FP509::new(
        0x0ce18b2e62c08386d9a58089433ea64b,
        0xe53820e4fde4957b08def0f0220de58e,
        0x502f1810ddebd0c7f3f848d907f1749f,
        0xb60fdba91bfd8b085935f6ef0e975110,
    ),
    // atan(110/256)
    FP509::new(
        0x0cfc98330b4000c6fb1f706d78cb034f,
        0x732be931406f2166b08f8bd2c088f033,
        0xd157110dded8200b9ebd6b69ce3cf3c4,
        0x0ab5f4b31e7f8054e916f7448369468e,
    ),
    // atan(111/256)
    FP509::new(
        0x0d17919f2f29858c0a6afbcb843582a7,
        0x8ce510320906369230cd21fb7bb7c0b1,
        0xd8befbbc3daba03e9fdc8df3f558b12a,
        0xab384c95df8ae7238d0ac5da9ceea739,
    ),
    // atan(112/256)
    FP509::new(
        0x0d327761e611fe5b6427c95e9001e713,
        0x66e0040dd2161a73c26677f8f13dd7e2,
        0x82857e840bc4483b4f7968592537f6c5,
        0xc372498dbbacd5cc9c22a0665adf405c,
    ),
    // atan(113/256)
    FP509::new(
        0x0d4d496acf4c6784b90b7fdd7605a94e,
        0x187f23ab581a8ba4d7981f932f01104f,
        0x680ef12522fb8b0b294d8d68614b13a6,
        0x07ad89a2c5bdfa160f7010df7297e9e2,
    ),
    // atan(114/256)
    FP509::new(
        0x0d6807aa1102c5bf89867123a5cfddef,
        0xc1486b0180295d385f27f3d45c7b189e,
        0xd2015d2bb71036d51a0e7ea62c5dcb8f,
        0x3d4f59994bda5cf281c14dbd3864cefc,
    ),
    // atan(115/256)
    FP509::new(
        0x0d82b2105749a1cd9d8d739397554d75,
        0xdfe6b55e49af69234de2726720e7d870,
        0x5bb94400396399bc3e095cea523f1857,
        0x0c2be629f7218d8341e550f0686e8715,
    ),
    // atan(116/256)
    FP509::new(
        0x0d9d488ed32e3635c30f6394a0806345,
        0xd55481118bfb7044cc8d2269f5943675,
        0x5e16830f1c2376a0db119f61571031fc,
        0xa2412b76b4924d9ae84fd9a2fba39b35,
    ),
    // atan(117/256)
    FP509::new(
        0x0db7cb1739bf7df1ad65cdb3f0c7aa19,
        0x19af293f0cdfe813dd391b3f0ba08af4,
        0x826c44b3f1ef6f0ff1fe07e54e700bdd,
        0x9fe2b2192b7b0f2a5c65f854deee12c8,
    ),
    // atan(118/256)
    FP509::new(
        0x0dd2399bc31252aa334368dc95fd2539,
        0xd8312f941a29ae6a633ac5cc75d49060,
        0x18d5b89897e9a2b3360e2090a8a6a5c9,
        0x22e090bd39bd3370189c3476814e4b65,
    ),
    // atan(119/256)
    FP509::new(
        0x0dec940f2940c8d69d578ca129336cc4,
        0x13697bda2e7f8d5cfd191f536b0bed9e,
        0x89756a05c39b35ed8340ff5be84406e1,
        0x4b6d29e98f90fc0e33bf61644c72c9da,
    ),
    // atan(120/256)
    FP509::new(
        0x0e06da64a764f7c67c631ed96798cb80,
        0x3b614bf2f5f3eb823950b3ebc58654b7,
        0xaad5715a4349a89309f6269130aca39f,
        0xad728e28ca893bdaca32ce7ce93db829,
    ),
    // atan(121/256)
    FP509::new(
        0x0e210c8ff88f5b49d0417969822474e1,
        0x0d1b7383deb854c6c23ed02530100711,
        0x9b6b1dcc0864a17474fd4ddd7a4fc5b8,
        0x87afee5570e6169912473a80294bbe6c,
    ),
    // atan(122/256)
    FP509::new(
        0x0e3b2a8556b8fc5168e8011f78370fd4,
        0xf7f162dccd5cfcacba6b54695d3410cb,
        0x0d7d4ace4d663948a3f011adcef78d5d,
        0x69894515ccb0cc1917aac638c8667853,
    ),
    // atan(123/256)
    FP509::new(
        0x0e55343979b18d819506781636f47ed4,
        0xeac32730b30f25cc87c71e2a4d431b75,
        0xafc43c984245c9771ffe292703244f7d,
        0xb50f3f36578957cf9a82db60d05c5d84,
    ),
    // atan(124/256)
    FP509::new(
        0x0e6f29a19609a84ba60b77ce1ca6dc2c,
        0x864b24e873cf283b40b03c10341eb3b2,
        0x9d6ae7b75c116f87a67a19d0ea5871e1,
        0x7507dd7f52416b87922e87b4999d7544,
    ),
    // atan(125/256)
    FP509::new(
        0x0e890ab35bf956b7ccf06b10d8605775,
        0xb9d767ef9c0ba871d5dd6774b1fec1d6,
        0x0760bec80fea523d9665d78c92f1def6,
        0xe7766959f24807d7d8f1eee6bbb93fc7,
    ),
    // atan(126/256)
    FP509::new(
        0x0ea2d764f64315989421163ef92d3494,
        0x6d35d282cd58a491f3710298287d1494,
        0xe53da3b50a6b6a81a83b495ad1d12009,
        0xeb872b1d2f56495a89b2ab270dee7096,
    ),
    // atan(127/256)
    FP509::new(
        0x0ebc8fad09137a6bfe815a49b589fbcf,
        0x3968b69a5bffdff91e2c2b5e803a56ab,
        0x28df8f4f1c7bb09c2541b0d2831c7627,
        0x2affdd1ca0f763ba432ba3e9f9365d28,
    ),
    // atan(128/256)
    FP509::new(
        0x0ed63382b0dda7b456fe445ecbc3a8d0,
        0x36e141587261cdf00e2cf16e6e962470,
        0x9fa9c5917892b516c87c812f8c6a4618,
        0x2cee1e80efd3c0013d6d85e3686228dc,
    ),
    // atan(129/256)
    FP509::new(
        0x0eefc2dd8134ba143bbb0c0a1225c299,
        0x6cddf6a61060e3b652b77dbff5e332e3,
        0x50b24228337e673d615f1b838805e991,
        0xfb509de80120f8fe33c1d81dfccc5230,
    ),
    // atan(130/256)
    FP509::new(
        0x0f093db583a257f6a7b3ce3395aaa30b,
        0x78f0e8afdebfc0ec818e2cf4e9497929,
        0xe8640f4c766fb69e82e7e9a7cd295ee5,
        0x93aba497580a8753d080c2ac71e603b2,
    ),
    // atan(131/256)
    FP509::new(
        0x0f22a403367a8d04e8ea84b3a63c7742,
        0xb623d21b42baf3ce1f1e5230e960aa24,
        0x13361d314e6af50a61389bd2776b629f,
        0xb2c3d7fab01e07d98a627ef904784f81,
    ),
    // atan(132/256)
    FP509::new(
        0x0f3bf5bf8bad1a21ca7b837e686adf3f,
        0xa2f20a0b8805d8e47d946887a4fd133f,
        0xff0f41a019ea7a8133a0d017926e26a2,
        0xa09d9a1d197c61bb8d74af4b87cdaa9a,
    ),
    // atan(133/256)
    FP509::new(
        0x0f5532e3e7946212fb2ceca3bf056a93,
        0x4eb3dba1e0061b7905f60d087018fc7c,
        0x8410579800323bad75db7ca5f1428579,
        0xccb775be266b010d2a8e9e43bf0ea017,
    ),
    // atan(134/256)
    FP509::new(
        0x0f6e5b6a1fc21a701b44e67bbd8e264d,
        0xc5810ba8c365a3445900dd9ddb30b292,
        0x7b3bf452a230c81b3d449fc21dc8a596,
        0x6ab5c37be35b7e9b55133310dc7355f2,
    ),
    // atan(135/256)
    FP509::new(
        0x0f876f4c79c9e7da17eaf069ced77ca3,
        0x07df5881fffd31bf5b7f2ff9cfb8373e,
        0x412323cb06312545da29ba9ea08ff160,
        0xf0a5f124704ebde4b53422a5c7052806,
    ),
    // atan(136/256)
    FP509::new(
        0x0fa06e85aa0a0be5c66d23c7d5dc8ecc,
        0x2100c92df8b8b5583632010fa1208266,
        0xacd9cd7c29184d88625becbd986bc101,
        0xd907525b1556078fa42b7021f4db9320,
    ),
    // atan(137/256)
    FP509::new(
        0x0fb95910d2724a8a44e107bf5ba0094a,
        0xf73699145e4d7bfb948fefce1fea6e3f,
        0xf9a015225af546ebefdce1231b9e9c7b,
        0x2d9b2ee3f38a46937629b976f6b509ea,
    ),
    // atan(138/256)
    FP509::new(
        0x0fd22ee981492c46b53847ae4e96a621,
        0xdcd69ecd96a8250e84c34251d320e7ce,
        0x33b72855e9b31297c29c0bd83c70bf90,
        0x504130aa73110eef3caa0724514d53ed,
    ),
    // atan(139/256)
    FP509::new(
        0x0feaf00bafefc193a87b1ec49b153ace,
        0x53f276b5044514e062fee9001fdb472c,
        0xe355751584c368cb716573884b314e2d,
        0x15608973386a07dd0e452b39d69fcb98,
    ),
    // atan(140/256)
    FP509::new(
        0x10039c73c1a40b9334dad8d8a9b270b2,
        0xcd2490c64dfc5c39805e4a7df2c41719,
        0x0869cdbe54f42b876ec9cb96388abbd2,
        0x1860a249204afc2e834c996449df7ef4,
    ),
    // atan(141/256)
    FP509::new(
        0x101c341e82422d4f6d13f32a43b77256,
        0x35ae78e10536102e81fa2a34d783336a,
        0x862136c6d993e4418f1f0a4a0d7856ce,
        0x1e73dea224b9bb6a34d01de940ee44c7,
    ),
    // atan(142/256)
    FP509::new(
        0x1034b70925048831f3671b0aa81fea86,
        0x3928e9e40c6176eb86247ea2fcabc0f0,
        0x2be76b9d69b682bd3b621b0d805de094,
        0x6a4f4d80fa5e43265183a12f7081a8ce,
    ),
    // atan(143/256)
    FP509::new(
        0x104d25314342e5b8de726ef4e263ad44,
        0xf77fe87ff101058da7808557cdda3353,
        0x94ec803b0a21e677a4d56a336324ee73,
        0x705fbbda7a2ba89376c4488567350fc1,
    ),
    // atan(144/256)
    FP509::new(
        0x10657e94db30cfc5496d41396c34a2b8,
        0x1e22ab9b0ee9bbf78ab8d7b9e3cb6cba,
        0x087afac19a9e27e9d05877ab7a97bb84,
        0x91c50e3eaa7ff02fadfe3d52fe855e70,
    ),
    // atan(145/256)
    FP509::new(
        0x107dc3324e9b3836e193c088a158e724,
        0x192d74db8f2f73c139a8e0cabc5cfff1,
        0x2c0a3ba4a61b27c0aa8c2ab2eb420f5b,
        0x028ff8d229055745fc3f7968ac713760,
    ),
    // atan(146/256)
    FP509::new(
        0x1095f30861a58fddbc9bebd4ac1a555d,
        0x55d48e420b8ada98b3c7e853f2348e3c,
        0x4552bdc48d101dc09a934fca6478891d,
        0x1293531d8254673725c30ec1ae127bcf,
    ),
    // atan(147/256)
    FP509::new(
        0x10ae0e1639866c20eb57e5bc88b4a7c5,
        0x8eba920ed806f4304e56b40e69cf10da,
        0x1747da14b3000a665570025f39727890,
        0x94e24668caa3eff4913f6d156d78790e,
    ),
    // atan(148/256)
    FP509::new(
        0x10c6145b5b43da0cba7d09daa027946f,
        0xb2552a1b528aa65215d17db1339ea998,
        0xe8bea8ca337acf2a98225ea3970f626d,
        0x7ed4b6773d758288613276f2d72a2b92,
    ),
    // atan(149/256)
    FP509::new(
        0x10de05d7aa6f7ccf92f69c75acaac457,
        0x4f8ea121a67872f62411303b8fc94c3a,
        0xb91ce99923bb5a7f6a19c2d59e80b0da,
        0x7c95c6d67faa06dbfecd5fb1198d89a6,
    ),
    // atan(150/256)
    FP509::new(
        0x10f5e28b67e295001311b17ec990d028,
        0x537bad7f3e2a1e6585bd538463e851eb,
        0x1454fe9f105c84f23adbad404660d24a,
        0x575b9fd667a8cf1e9434bba5eabd190d,
    ),
    // atan(151/256)
    FP509::new(
        0x110daa77307a0d5a70cf5131ec15f3eb,
        0xb79c264bf318059ef8f6b0e7234bd394,
        0x33a820a726ffcb49ba1a1791d4cc6c10,
        0x76782aa168be5aa8643cb2ef84815732,
    ),
    // atan(152/256)
    FP509::new(
        0x11255d9bfbd2a8f6a1288f1f88e5695f,
        0x71f7cf73f73bbecfe5181445ccbfec01,
        0xc6240bba1c72deec145a11695bf455ca,
        0xfe8466fd1656a37bb331ae789db6a1f0,
    ),
    // atan(153/256)
    FP509::new(
        0x113cfbfb1b056e4c4439bf0fb4df3138,
        0x165cb3ddf54ef9833e801a05f4eb64a3,
        0xe0ebd43d46e24c899ec0fe685cf0854a,
        0x9fe286a15c965034ba87ec9618fc74a5,
    ),
    // atan(154/256)
    FP509::new(
        0x11548596376469ad160eadd030f3d087,
        0x3fcedefd99f77e7ad6bf804a9b894407,
        0x68b032dd37d03b34a4c44aa7392cf452,
        0xbe5b1a53752fa0a6a249326a8ede7176,
    ),
    // atan(155/256)
    FP509::new(
        0x116bfa6f5137e132c0dfc2837a690c69,
        0xb329259ed74057e9a7ea932a5282022b,
        0x89a6fd15b380d6226533ead813970201,
        0x3696acac86d5c7cb28e1b01a38f06591,
    ),
    // atan(156/256)
    FP509::new(
        0x11835a88be7c13718873b0030c040b24,
        0x1f1c4889206239c0d95245c7c806c9cd,
        0xf706816d149526984545c2937115a8bb,
        0x18fbb367ebfe3cceb89d1197847e504c,
    ),
    // atan(157/256)
    FP509::new(
        0x119aa5e5299f99967e4f4e9c1f2c0567,
        0x70e13a1e4291f154c59b5e77d0db94fe,
        0xc10589e7a80c8786409845fb031e1a8e,
        0x055cd1dbd99784909287eb328a107518,
    ),
    // atan(158/256)
    FP509::new(
        0x11b1dc87904284ede173751070d71459,
        0x9bf8570c2b96db74667a79d0c9271e70,
        0xd1c18457bc892ed9cf153fff624027a8,
        0x789116ad190ef56e13e3069079b4c202,
    ),
    // atan(159/256)
    FP509::new(
        0x11c8fe7341f64f2517793abcf253f1d9,
        0xbdf39d83510bcb3752994ec6d3531220,
        0x55bf273077e61e6b529d5353cd72408b,
        0x96fda69bb3b0dc9cf622e324700f7bd9,
    ),
    // atan(160/256)
    FP509::new(
        0x11e00babdefeb3f36b906bc2ccb886e8,
        0xf2314cfc0ca566eb9ea7b48da26713d6,
        0x46cd3b9a7231ae256cd9afba60e7ad4b,
        0x05389da3afca475986b1ec8814866628,
    ),
    // atan(161/256)
    FP509::new(
        0x11f7043557138a2d8cb3edafba8e26d5,
        0x36cd1b0ee1c394144cae70cfe85547f1,
        0x36675b71e62c6ded860b1c85e2c07ceb,
        0xfba5c889f2085e251d1c85af8c7c3b1c,
    ),
    // atan(162/256)
    FP509::new(
        0x120de813e823b1a1b8a2b05122f0a156,
        0xb33e149088335fd4dfb2a9ce507f0086,
        0x9a88ad1dfe1adff533628dc9db1fe41f,
        0x83913054a9cd9c0cc1c65dc439c02e52,
    ),
    // atan(163/256)
    FP509::new(
        0x1224b74c1d192a75b4f7e22983101ed4,
        0x5616a3a2640a2473a4b063596517a2a3,
        0xd3e9e5fc218ef1ca35ecaeb0547b4a47,
        0x1d81fceb279b586bd166e0e11d044eec,
    ),
    // atan(164/256)
    FP509::new(
        0x123b71e2cc9e6a1c421c9f38224dc043,
        0xfb32bd1a3f86a686eceb1e41c0ddf7d2,
        0xed0999b1bdc91872079ca86267d61f68,
        0x47329b5be44cd8d213edcf2a7db4e582,
    ),
    // atan(165/256)
    FP509::new(
        0x125217dd17e50155aaa2306d9e6f2fae,
        0xdd4a167ce43797e5a496f7db90db8f74,
        0x6fbbd09780e2dbd81e262b005d714509,
        0x65e7f55a3be1ee1f204e1299bdb101be,
    ),
    // atan(166/256)
    FP509::new(
        0x1268a940696da60e89a4502639e63930,
        0x389b69f25852372fb5d55b34ffdd6ee7,
        0xf7dba714c67c7f47c607d08d075cd759,
        0x3bcd9050b4220dc82445b673179dafe2,
    ),
    // atan(167/256)
    FP509::new(
        0x127f261273d1b350efcd8547767c5d9b,
        0x4ae7b805feef8daf1ad486c8e95e5f78,
        0x857f403ac04ebbd74f1fd3e000e8fdc5,
        0x969507c5e143dbc73d23368d49545470,
    ),
    // atan(168/256)
    FP509::new(
        0x12958e59308e30dec3189e727ef1465f,
        0x1e715c1e8f528bdb458d73ef100638f0,
        0xc6cdb79c372fd00dd1255d7704685c04,
        0xe60ff24473f919ba47d4affc01e48eda,
    ),
    // atan(169/256)
    FP509::new(
        0x12abe21aded07370a30006b408b87622,
        0x2e91b7c8e01d8d57edd0208fad8aa1de,
        0x6a6bc0e607eb6d8d8bf55920a7ec4af4,
        0x2fa92efec0fc44efc040276cec7a2bff,
    ),
    // atan(170/256)
    FP509::new(
        0x12c2215e024465fad1fbc9716d07c4d0,
        0xc935e7723290f06cc4b2b422fcb39878,
        0xc9f725ffe7cd585462fafe0dee5a08bf,
        0x4a3ac474ccb6b9ff4ae0eb7b74602fb8,
    ),
    // atan(171/256)
    FP509::new(
        0x12d84c2961e48bc1b57beb9235f48555,
        0x03fa527e8c4425fddf78417c9dcb5cf8,
        0xa09a9bfd00f6a9e5b2ca9dfe5bcf1415,
        0x17022ffee9b4c2f04f7301b641a5b788,
    ),
    // atan(172/256)
    FP509::new(
        0x12ee628406cbca71757a7fc33e35d6d6,
        0x52f31b64e80e0af1c3f5e41550e98c9a,
        0x84764818b70e37ec00916348d9924242,
        0x902e7e7656dc54cf02bf074d5587d4e0,
    ),
    // atan(173/256)
    FP509::new(
        0x130464753b090ad831db3c4bce19b741,
        0xda29f8b04c97bdff02903ca3f703bc6f,
        0x8cd972bff4ae5b2ed97aaabf2855ae52,
        0x130c863542d06723c33704ab89a74703,
    ),
    // atan(174/256)
    FP509::new(
        0x131a52048874be5032ae1e9eb0912548,
        0x5fb98211f333f81b44be7c4b72649814,
        0x10949180b16906ceaa3875baf0db0784,
        0x41dfdd5a60e9c2a4db14756c1df6e328,
    ),
    // atan(175/256)
    FP509::new(
        0x13302b39b788565774bb61eea0acf15a,
        0xcb27d1a36816f715faa438db705a8540,
        0x9acebb1db538bb3e0418c44828459aaf,
        0x215815362307f3a2d5aab29749bc614f,
    ),
    // atan(176/256)
    FP509::new(
        0x1345f01cce37bb440844df1c4409fe77,
        0x9b5c8de0c2e913ab8ef0e9a3cea75fdc,
        0x8999b7fcd6e95b11e22c415c52cd7b53,
        0x8131cabbb3889f503d3ce1a522811c36,
    ),
    // atan(177/256)
    FP509::new(
        0x135ba0b60eccce78ee866e4da2e56e13,
        0x2d807a3b13aff3b36c876a92e87d5d73,
        0x5892dd9ed69c5d41d55b38484142cf59,
        0x4430f015d6c1513cdb3d8a19fffec528,
    ),
    // atan(178/256)
    FP509::new(
        0x13713d0df6c503f5843b0fe74fc96dba,
        0x78bb5581db56ecf35c0354e684b92aa6,
        0x0154afcc94c1ab0a1c443b9e123a9e33,
        0x5b9debdc5942b7823dce783b19a91a80,
    ),
    // atan(179/256)
    FP509::new(
        0x1386c52d3db11e921c78d050657f7b42,
        0xdd2526002c003c6bd597dc6a03a87c4d,
        0x18ba2806c99beb986b03d4b2a178f86a,
        0x60798e9031123be1c5e6c82fa6b389b1,
    ),
    // atan(180/256)
    FP509::new(
        0x139c391cd41719b73ef3389d02e99e23,
        0x8b4558d4764dcf27de3cae4bac1a59fe,
        0x58b59b6a4d959b5f469ac795bae4c81d,
        0x5e04c633f46d9de3d52bc34b44a59ac3,
    ),
    // atan(181/256)
    FP509::new(
        0x13b198e5e2564ada0dbbc1f15aa841b2,
        0x7fdd7f6025cab0b77d70f8c76bde67bd,
        0x8c3d46119b5032dab0dd09e82b560771,
        0xd53331ba57014aed795b081c1e89037c,
    ),
    // atan(182/256)
    FP509::new(
        0x13c6e491c78dc487aebdac0bd1625173,
        0x8729ef590af93227d193dccbaa9659d0,
        0xc2c3fb4d46c1cd450254e06ee96444de,
        0x5d96d5996db2a1d276ee27cf1bcf688f,
    ),
    // atan(183/256)
    FP509::new(
        0x13dc1c2a1885044b398dc3d3a5c3bfbe,
        0xb24b413fffb17286f5e13db52a2c63cc,
        0xd22839a701b16f35b0eb9982d4a46d55,
        0xe2937b6adc7f89c4acc90c17ad1ffd7a,
    ),
    // atan(184/256)
    FP509::new(
        0x13f13fb89e96f43d9f16924c89e0e03b,
        0xf3ae87a37900359540146b78438de9bf,
        0x7b6ed1060ef0b1e8c506f41bacece951,
        0x6951b9bf7db133b7a29282dd258c2479,
    ),
    // atan(185/256)
    FP509::new(
        0x14064f47569f489549dc1b903537cb4e,
        0x12258ac7533952de2ae1b5899fe0ab89,
        0x33242aa130a1726ecf887215de3ba7cd,
        0x7bced99a69c634be151797398f8462ad,
    ),
    // atan(186/256)
    FP509::new(
        0x141b4ae06fea4113d60a53277651d8ad,
        0x072cc5d97ef8c6e08677848655a5d104,
        0x7bfc2725cba587706753550d0a498053,
        0x6e1b6efc37c0ec15a0209754a5002cf6,
    ),
    // atan(187/256)
    FP509::new(
        0x1430328e4b26d5bb3a9bbe3d1985deec,
        0x7e1ea6b2602fc7264206399fd0b56e18,
        0xf8a9add7156d696a534a1a26d4096104,
        0x0c1453afa2090b1df12c84267bf777d8,
    ),
    // atan(188/256)
    FP509::new(
        0x1445065b795b55c1125fd3810c6f5e1c,
        0xba10828da9faab7c82b423e76a045af0,
        0x668a30bd259fc3a67e378503dcdd22a1,
        0x32d6058540a27a936f45fd91931c8c2e,
    ),
    // atan(189/256)
    FP509::new(
        0x1459c652badc7f4665a63a384d6f5127,
        0xa7eadaaebd979be14aaea602ca91320b,
        0x565473d616f8f6a1eee041159e46e74b,
        0x69f0fe7d28c61b09da303dfb5b64cbcb,
    ),
    // atan(190/256)
    FP509::new(
        0x146e727efe4715ec6464e47bb3373dfa,
        0x706ef43d8ae8eadb5e3fba8ce2a336fa,
        0x846c7447e99008b4d44a847d36810683,
        0x4d6671fa66a70f1809036b4ab0c24097,
    ),
    // atan(191/256)
    FP509::new(
        0x14830aeb5f7bfdf2ecd4ccc4dac6669b,
        0x1a8769c0c751c675f4deebd16ceefc29,
        0x43f182cf06eb30e6ad26b952a55f4e5f,
        0xda2ca54a8a22aff765e786be9c58bbc6,
    ),
    // atan(192/256)
    FP509::new(
        0x14978fa3269ee12483350fe548afb593,
        0xdc7e10d13dd6573ce4290cccb1989de7,
        0x54fef6fb72679709eaec440dda7a0496,
        0x722e85999cf0450b591d41948a901eda,
    ),
    // atan(193/256)
    FP509::new(
        0x14ac00b1c717626c839c8e0ae4007d68,
        0xc53de3d9419a1cb1b3639ff6b9bba22c,
        0x095208d8bd2a91125c136668537ede7c,
        0x76d36acf166aff1189d76fa201efd1c5,
    ),
    // atan(194/256)
    FP509::new(
        0x14c05e22de94e48fd4f83d8344848a3a,
        0x88c84c98b922dd1d1d8eb2487ae5edca,
        0xa4da5cc26008ec3568be89f5ba8d9a17,
        0xc7d7e2a043ee55e353ea0df0702ed81e,
    ),
    // atan(195/256)
    FP509::new(
        0x14d4a8023414e81e3a891daa88b01dc4,
        0x9430e552b85da27966631465622a4cad,
        0xb99373aba26a8aac43de3d92707a6de2,
        0x4b605362c1c186ea8c0d152e184cb0c6,
    ),
    // atan(196/256)
    FP509::new(
        0x14e8de5bb6ec04528cf6facde5ae9c03,
        0x243702ee9b0f46d49d67fcbf49294264,
        0x47388101edbb7704a13ec8681259ec3e,
        0x9e70b617551ddc0e48be8b287384d093,
    ),
    // atan(197/256)
    FP509::new(
        0x14fd013b7dd17e3aa27e7cf84969a282,
        0x561a42595c46b5574acdcfd17e1a232b,
        0x3d385fefbb97b4c3b8860f16a25a9c77,
        0x083b4e1370797c35ad59189ef9e6c585,
    ),
    // atan(198/256)
    FP509::new(
        0x151110adc5ed81247b9ad0654c7bc3dd,
        0x1c2f5694b5e978cc10f042363e4afa03,
        0xdd94374ba891896ab3dbc7a9848fc8ff,
        0xfbb06a8720baf349738ed7c29c58fbf4,
    ),
    // atan(199/256)
    FP509::new(
        0x15250cbef1e9faf563242e6ebbc8373b,
        0xee7953346a4e8c5617b7569b36c4e13a,
        0xd511a54dbea51f0fe517f8e357d8484e,
        0x3b33197743ce502dba3d4b41df306952,
    ),
    // atan(200/256)
    FP509::new(
        0x1538f57b89061eb9122d5096b7cf267e,
        0xbf32e2cabc84f7e38129e0074fb7eb89,
        0xa8b263ae86d439433fef5b3360ac55bd,
        0x8e5d64b3ff0863dbd0b8cfb4c0c2a363,
    ),
    // atan(201/256)
    FP509::new(
        0x154ccaf0362c8f628c91fe3d0f05598a,
        0x2e59bca90efdea117237ad04feed3a71,
        0x1d853cef4205c6a735789d2159f48e2d,
        0xba8c0f4d6f3a508fdb09ab5d65dcc0cc,
    ),
    // atan(202/256)
    FP509::new(
        0x15608d29c70c34664e73c37a022001fa,
        0x9f43b73cf1cd3c4e13e643f94b7946b0,
        0xaff4bf715c5d936da50d592361c5802a,
        0x3344a8565116ffaf195636154c65c021,
    ),
    // atan(203/256)
    FP509::new(
        0x15743c352b33b9857fcb2cde0568d3f6,
        0xc06e516e1c1a58ab6ad440c40549a11c,
        0x829fea739ce7f73836800116f7490a72,
        0x19f0ab75ac316a17480030be58276c21,
    ),
    // atan(204/256)
    FP509::new(
        0x1587d81f732fbad4346c4e74ad5f51de,
        0x8307b6a80d8840d1970a96b9f4fdbe0a,
        0xa798dd66a394469ff373557a2de64b5c,
        0x69432bb5f877bceffa7fb6eedfb146af,
    ),
    // atan(205/256)
    FP509::new(
        0x159b60f5cfab9db93ecefbf650ed3e49,
        0xa32a37706261baee5e861344989c4a27,
        0xec902a42d3bdf6a9964f0a8547345733,
        0xff21517fbffda16408c280cf86d4931e,
    ),
    // atan(206/256)
    FP509::new(
        0x15aed6c59095175cc4bdc52a50d9b6b2,
        0x2a53e9e86b58fec726c37ae7a64f0c17,
        0x0408200e6c8d0971f1a1b28f58212931,
        0xcf764be5cb27b4752a74613e8df759e4,
    ),
    // atan(207/256)
    FP509::new(
        0x15c2399c244260b390ac586a532a2a95,
        0x7828ed0cd8855820803c5b9498be3056,
        0xe36e7095f4905c84b02761f2b3913604,
        0xb80d1fbee219da67a98ad03d085b847d,
    ),
    // atan(208/256)
    FP509::new(
        0x15d58987169b1810028e4bc5e7ca40e1,
        0x406e8568c87dffc4c0880f28f889fba6,
        0x12bdc1fa3dde3e7af8cb0dfc3ba8735e,
        0x67fadef0a76e8d85aa60fda978a7859c,
    ),
    // atan(209/256)
    FP509::new(
        0x15e8c6941043cfde81148375c3209f22,
        0xc8e6df8897f76b8d0dbabcefe9c7412f,
        0x8ef7ab3536d6b4d1870877b9687b143e,
        0xcef1e4752c11b33c20d2a31de3384f49,
    ),
    // atan(210/256)
    FP509::new(
        0x15fbf0d0d5cc49f259817ffa475415cc,
        0x4fc822633e4a68f69245cde62e50215e,
        0xe54f63ee122de0cebefed08e649a25d7,
        0xb62dbe0f1df34483597e1ae8a1cc439e,
    ),
    // atan(211/256)
    FP509::new(
        0x160f084b46e05e8911e599aeb9b39b5e,
        0x54b61e8542293ea2e08d2d4be27ac4d9,
        0xa3bbf0a9bcc90214f4bf1f788e99ec4c,
        0x58e88916a7f9ec7bfdc534d41d8c561f,
    ),
    // atan(212/256)
    FP509::new(
        0x16220d115d7b8ded487acaf1173ed4f6,
        0xa13c5051a9bf3c38eba7daad799cdb65,
        0x640a2217a084b29f8f21761357bf9488,
        0x2628e2d7c23358ede741d7e5a43f6dc9,
    ),
    // atan(213/256)
    FP509::new(
        0x1634ff312d1f3b674bcc57cad65f3d29,
        0x475c783ece2dd4ad4cd4c6ba5abe05dd,
        0xc5c86910a5b635ec60c405601a5049ba,
        0x2a557e6876e14da6a933a182fd4b5a4e,
    ),
    // atan(214/256)
    FP509::new(
        0x1647deb8e20b8ff09afdfee2d7186648,
        0x565477b94a6e699d47862ef4581e8175,
        0xa4a6538f309dee4f073969b76bfc9c18,
        0xbd5a0629ed86805a6346cceb2a088449,
    ),
    // atan(215/256)
    FP509::new(
        0x165aabb6c07b02e86c64c50cd8d73b89,
        0x745a108fd20e39c1dd48d3787141289d,
        0xbbfcb6f24c7c7751f7bf74b3d8f87ed8,
        0x58c78b137f8ac8e72eec75ad09af65a7,
    ),
    // atan(216/256)
    FP509::new(
        0x166d663923e086d22b20282e888c5f27,
        0x3c8d3fe500f07edfcea3cbbd8813b9a3,
        0xc875918bfde9314aee1808b53f94e3b1,
        0x819fb8d393d2ef5f68f0be8eda0735d2,
    ),
    // atan(217/256)
    FP509::new(
        0x16800e4e7e2857f38acaf260b787e2ae,
        0xcd34a612a5ffa1fee8780d2e1d46ff8b,
        0xf4dc3df3b18899d46ebe1c010fba9d09,
        0xe7a557a2b40e7b9824471926780dd24e,
    ),
    // atan(218/256)
    FP509::new(
        0x1692a40556fb6a7652e56a3a8b2e394f,
        0x0e84de7771cb67cea15077ed2aa3886a,
        0xce90d5ca486b3fb9e18562869988e1fc,
        0x0785b28f113954aada7fb57a18049dc9,
    ),
    // atan(219/256)
    FP509::new(
        0x16a5276c4b05758252698ee56587f313,
        0x7813315e3f56ef1f2f8d8b4200377ffc,
        0x4c35c21e1fecdddc1e8a7b8f713ed4e0,
        0x1344bdd558ac64e534611c3f76d6f857,
    ),
    // atan(220/256)
    FP509::new(
        0x16b798920b3d9895ff1e79dcebc720fa,
        0xaea42de8f67123c0f19bb558319d52a6,
        0xbcb06a1e6df51aaeb745941dfdddcd9e,
        0xfdd1618b82f25ab955d1aaa604dda396,
    ),
    // atan(221/256)
    FP509::new(
        0x16c9f7855c31983813bc537a50191416,
        0xddcfc75bcacf2294db8f1cda5ab39e0e,
        0xa65e6817eecce42d6e463bcfb09a4000,
        0x6a631657cdc25654cd4317ac3589a064,
    ),
    // atan(222/256)
    FP509::new(
        0x16dc44551553aef203bce5463eb90c93,
        0x015af43d8e767fa2be4b6aa2ec45c80c,
        0xbc60643526f5a16d846b4e8ad88aa208,
        0x8f0a284fff0a3d338edd11a817a73613,
    ),
    // atan(223/256)
    FP509::new(
        0x16ee7f10204aef5a4bba8c1989c7df05,
        0x1f23492aa1acfe92f37f7b12dc1c7b2f,
        0xacf44182bdd7de20b07f2e2092ab2ae8,
        0x47b27f15a56004cdfe27675582787b18,
    ),
    // atan(224/256)
    FP509::new(
        0x1700a7c5784633ce7965b4aa42148887,
        0xa7af5d982298f350140242abec995ca6,
        0x08cabe0cb9d93030493893d8e4cd04e5,
        0x9d18a7c428700f038704afc8328bd19e,
    ),
    // atan(225/256)
    FP509::new(
        0x1712be84295198573640cdf62205eee2,
        0xeb49ab483546de26a5002ec79f50a895,
        0x5b8c6cfb5db5fbd4d6ee5a6e98ea0308,
        0xa71ae77b8ac9bd3bf6fc051368fa09e6,
    ),
    // atan(226/256)
    FP509::new(
        0x1724c35b4fae7b0ca45996d9a4cd788c,
        0x00bfbd067e5179e544f3f7bc463bd7f5,
        0x50b41cb91b5b77200b3f84176b9c3c44,
        0x8d4afe1d83f8ff604189997eb5238856,
    ),
    // atan(227/256)
    FP509::new(
        0x1736b65a172dff2eebfa0d5125a201ce,
        0x1e03e0da9aba1ef405c89cce3583c43a,
        0x3fd3d182cfe29dd386dbba4fe19bae7a,
        0x3253c28951a50848b3ead1b3284d3025,
    ),
    // atan(228/256)
    FP509::new(
        0x1748978fba8e0f05eca9859621285c70,
        0xc464508a9a8a946860ce5f9faa3dd9c4,
        0xe83483f1a1ac69fd9dacc91ec6f8b884,
        0xb97c4bb12c5d4a1a5849ea30fdca8d35,
    ),
    // atan(229/256)
    FP509::new(
        0x175a670b82d8d87b92b131ca421b42a1,
        0x691994e621518f63db15d16d13124ada,
        0xdf0c07f7586e5a50b9b486625e1f7f11,
        0xdd31d01ea634330cd419c9a52ee7a2a3,
    ),
    // atan(230/256)
    FP509::new(
        0x176c24dcc6c6c046549546b720f22b5c,
        0xc2685bdfedf1ad8cbc58e329a0b6f28c,
        0x9205d72aad24fbac7125dbdd30d5c541,
        0xe7fda88e6b5c579536b140163b062f54,
    ),
    // atan(231/256)
    FP509::new(
        0x177dd112ea22c75cc9823f0434f5c7cc,
        0x4e81d355f4b33cfa50eee6b7241a9909,
        0xa87d6a0a7290cf7c162c31103ff1e677,
        0xb1115a81b2ddc466b943a4e46f58cf83,
    ),
    // atan(232/256)
    FP509::new(
        0x178f6bbd5d315e501a822600dd01f652,
        0x027f5703eb7495714462f2d76aa29532,
        0xa02190bf8ad308be7a82818f7a0a00cf,
        0x92417469ebab1b515329d330f8987882,
    ),
    // atan(233/256)
    FP509::new(
        0x17a0f4eb9c19a2113c67cd815f576bae,
        0x82bafad4b1492e2d17d037f098e3fdaf,
        0x2084b2acbe7d932555f57615a23e9b25,
        0xeeb5194f0ab56b5ee5f9083fe0f1ab24,
    ),
    // atan(234/256)
    FP509::new(
        0x17b26cad2e50fd8c5fc833efb81362f2,
        0x313f63c4d8e4de6254b275e1eadc9770,
        0x870948ce2b949eff73e7437fc24059b1,
        0x39ccbbe732488e7e456601f675eb1b91,
    ),
    // atan(235/256)
    FP509::new(
        0x17c3d311a6092b6ecf2cb4c0ca2136e8,
        0x6d776eda4ff4b16653857e8b2cc99977,
        0xcfc9770aac73d4d3d47911bfb8678e6f,
        0x2c8c817b2ae1e3fe0292b5a21b732a45,
    ),
    // atan(236/256)
    FP509::new(
        0x17d528289fa09355820878bceaa505cf,
        0xa02870683e27ab874e9fb5de2cab7e68,
        0xa8261ecba028d336329e679abdb81dcc,
        0x62963bce62c06ca0184de5955d675e10,
    ),
    // atan(237/256)
    FP509::new(
        0x17e66c01c114fd8df51dd227d1ca15b4,
        0xcded7fa4d44c0d9b02a23d0a5cee71cf,
        0x5d827771af7fae0b33003fcc892305f5,
        0x4c492eba8313939ba770b91792919bdd,
    ),
    // atan(238/256)
    FP509::new(
        0x17f79eacb978987f572a03ab48834496,
        0xb9c410bbea7268ab0d563ac46c966cad,
        0x4c12708b81bcdd0ac43b66381ec300ec,
        0xa34759dfb533f74ef20b662f523802f7,
    ),
    // atan(239/256)
    FP509::new(
        0x1808c03940694abfc33623a8256c61a9,
        0xab2a7175748d4a14aa91d205d73446c8,
        0x50dc83c9d50f0e1e042bfa1089101e21,
        0xc30ac8237d016aea84469d4e95d7a16d,
    ),
    // atan(240/256)
    FP509::new(
        0x1819d0b7158a4cc8113bac588dd25f44,
        0x9ab1dd0cf5de86dec6e1028cf42165a4,
        0xa89ff4a78a34c5eaf46958b9e90b78b8,
        0x475a06c6d592b996dbfd0d707320ac7c,
    ),
    // atan(241/256)
    FP509::new(
        0x182ad03600000528b25f9d249a484bd5,
        0x0d2b7e4f8e07c2d88130f7dd9a2aea0e,
        0x11adfb75b07067dcde53f160bcea2f0e,
        0x0e88b26cdab9c1f5afb0fc2dbe556f0a,
    ),
    // atan(242/256)
    FP509::new(
        0x183bbec5cdee2213107104ffc6c2891c,
        0xe30257209c76007e4cca7775702ae13a,
        0x2eb5f354c2c04764c08b1df57d5ae301,
        0xa8ae9668dc66f4348d2b092205bc8c94,
    ),
    // atan(243/256)
    FP509::new(
        0x184c9c7653f7eafcf93dc03eb82e27c5,
        0xf007351d8308f8af7f919ce519d1044e,
        0xcb60d123c25b824bb834eb0df69cc591,
        0xadda0ba64cae1f880b40f485c799f491,
    ),
    // atan(244/256)
    FP509::new(
        0x185d69576cc2c516b66e7fc8b8c3773f,
        0x5d2e1d81c1c9754cd49485dff52e48a2,
        0x395023ad401d1efde2de048ec93a4e5e,
        0x883e74f8dd19629c6b9260f233ed83d3,
    ),
    // atan(245/256)
    FP509::new(
        0x186e2578f87ae5408ac4dd73f8cf1fbc,
        0x1274e5e61a096fed554255d904318dac,
        0x9871d2375bedea4d01bef427c22b3ea5,
        0x9d1147f96bdbc2312b8544622c52ca56,
    ),
    // atan(246/256)
    FP509::new(
        0x187ed0eadc5a2a215eb5b2afe978c2ac,
        0xb9c954c47956c5f6aaf6e8d1e60c3e5e,
        0x79481013b04cc6f4590c50d633503ebf,
        0x663a3c2e7bfaa82b3c54e2fe6d8ae413,
    ),
    // atan(247/256)
    FP509::new(
        0x188f6bbd023118f669716d2a2ccfc4a0,
        0x52e325770c79707d523b0bd81e6569f2,
        0x6399936f996c75a3214626ca0871fea1,
        0xa748ab17ab207d64585b6735f11927e6,
    ),
    // atan(248/256)
    FP509::new(
        0x189ff5ff57f1f7aa919687a21793c044,
        0x29b4c436c0073069dfccd3b329f2fa3f,
        0x561faad288b8a26a540d1d619f08c784,
        0x449da362c000d79341316d953a51a5eb,
    ),
    // atan(249/256)
    FP509::new(
        0x18b06fc1cf3dfebc133b66a4924c7a3a,
        0x8d06fadb429657f7decdac06769572b9,
        0x337518a0d8c3fcd926931a04107575b0,
        0xb519008e77166002c711c383ccbe63e7,
    ),
    // atan(250/256)
    FP509::new(
        0x18c0d9145cf49d6fa901db710cca2c8a,
        0xadee96bb5142b8328884a63e6f347fe2,
        0xfe5b655768f429f44893aa460b9cd8d7,
        0xc3127fdeeb382b4f629224d5d124513e,
    ),
    // atan(251/256)
    FP509::new(
        0x18d13206f8c4cac9fce68aaecae9944c,
        0xdba2788e5c75ea93f4e47c0b628454c3,
        0x6deaeb6f88c8fdd1ed3f0df91ab1627a,
        0x59a9908bcaf5116af184824689ba0e38,
    ),
    // atan(252/256)
    FP509::new(
        0x18e17aa99cc05dc27cfaa9f7a13e57c7,
        0x01d3930dd5f36e0555bfbd2529082ffb,
        0x57a0d3f576b0918fd7739379fbd02c55,
        0x8e49a38cc82ebd73bed865f8ef9ae92b,
    ),
    // atan(253/256)
    FP509::new(
        0x18f1b30c44f1671dd1cab93933fd28d6,
        0x47938ec73d169a615a82d03673afd140,
        0xd2e50a32eb772a9d3657ca050436e7e3,
        0xdc1602aadf81fc3e89a7f6e52d696354,
    ),
    // atan(254/256)
    FP509::new(
        0x1901db3eeef1875a19979580f23dee3a,
        0x876fa537d8e10429908055c02e531494,
        0x8b8a819182334ac293443c4c0727c6d0,
        0x4dedffa44eb81e52a3d9488c1d29b0ba,
    ),
    // atan(255/256)
    FP509::new(
        0x1911f35199833b13ae8a0edbf521aa61,
        0x82cf6992bcf72667db310955cfd40301,
        0x74b22f9da03c7eabf61d10956e79d575,
        0x7c14b2304c88c6d9b41b0a09407c426c,
    ),
];

pub(crate) fn approx_atan(x: &FP509) -> FP509 {
    let mut x_abs = *x;
    x_abs.iabs();
    debug_assert!(x_abs < FP509::ONE);
    // Reduce |x| so that |x| = c + y and y < 1/256
    let (q, c, mut y) = x_abs.divmod_1_over_256();
    // atan(|x|) = atan(c) + atan(y/(1+|x|⋅c))
    let mut t = x_abs;
    // |x| < 1 => c < 1 => |x|⋅c < 1
    t *= &c;
    t += &FP509::ONE;
    // 1 < t < 2 and y < 1/256 => y/t < 1/256
    y /= &t;
    let mut y2 = y;
    y2.imul_round(&y);
    let mut atan = COEFFS[0];
    for coeff in &COEFFS[1..N] {
        atan.imul_round(&y2);
        atan += &coeff;
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
