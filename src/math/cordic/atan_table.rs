// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{big_uint::u256, math::BigFloat};

pub(crate) const ATANS: [BigFloat; 255] = [
    // 7.85398163397448309615660845819875721049292349843776455243736148076954101571550e-1
    BigFloat {
        sign: 1,
        exp: -1,
        signif: u256::new(
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    },
    // 4.63647609000806116214256231461214402028537054286120263810933088720197864165742e-1
    BigFloat {
        sign: 1,
        exp: -2,
        signif: u256::new(
            0x76b19c1586ed3da2b7f222f65e1d4681,
            0xb70a0ac3930e6f8071678b7374b12385,
        ),
    },
    // 2.44978663126864154172082481211275810914144098381184067127375914667355119587643e-1
    BigFloat {
        sign: 1,
        exp: -3,
        signif: u256::new(
            0x7d6dd7e4b203758ab6e3cf7afbd10bf2,
            0xd53fd481c459c1b5bd1d3b3e4a24d3b3,
        ),
    },
    // 1.24354994546761435031354849163871025573170191769804089915114119115722267427566e-1
    BigFloat {
        sign: 1,
        exp: -4,
        signif: u256::new(
            0x7f56ea6ab0bdb719644bcc4f9f44477b,
            0xc59cabecaecd6c917649abbefdb60bb4,
        ),
    },
    // 6.24188099959573484739791129855051136062738877974991946075278168986902672168040e-2
    BigFloat {
        sign: 1,
        exp: -5,
        signif: u256::new(
            0x7fd56edcb3f7a71b6593c96e071706a8,
            0x98ce0967acea596e1c3d4fc01e25c577,
        ),
    },
    // 3.12398334302682762537117448924909770324956637254000402553155862557964210194326e-2
    BigFloat {
        sign: 1,
        exp: -6,
        signif: u256::new(
            0x7ff556eea5d892a13bcebbb6ed463109,
            0xc036814a606dc40b2380beda26b08304,
        ),
    },
    // 1.56237286204768308028015212565703189111141398009054178814105073966647741764017e-2
    BigFloat {
        sign: 1,
        exp: -7,
        signif: u256::new(
            0x7ffd556eedca6addf3c62b200afbb024,
            0x1077b2e086f77a3034afc0193ab0e5d4,
        ),
    },
    // 7.81234106010111129646339184219928162122281172501472355745390224838987204533520e-3
    BigFloat {
        sign: 1,
        exp: -8,
        signif: u256::new(
            0x7fff5556eeea5cb40311a8fddf3057a3,
            0xb42b500b8d17800d41e8abb3f7672848,
        ),
    },
    // 3.90623013196697182762866531142438714035749011520285621521309514901134416395440e-3
    BigFloat {
        sign: 1,
        exp: -9,
        signif: u256::new(
            0x7fffd5556eeedca5d8957db5b6a7bf0b,
            0x3e0c5d75cde4abc491245693415f7b37,
        ),
    },
    // 1.95312251647881868512148262507671393161074677723351033905753396043108530313710e-3
    BigFloat {
        sign: 1,
        exp: -10,
        signif: u256::new(
            0x7ffff55556eeeea5ca6adeab02251ce8,
            0xf2409fee23880bec67783a2d83ee2689,
        ),
    },
    // 9.76562189559319430403430199717290851634197015810087590049007252267637520355086e-4
    BigFloat {
        sign: 1,
        exp: -11,
        signif: u256::new(
            0x7ffffd55556eeeedca5cb4033f79d4be,
            0x69b7875594262279d06b22bac1dcd381,
        ),
    },
    // 4.88281211194898275469239625644848666192361133135003037109403353487512136743276e-4
    BigFloat {
        sign: 1,
        exp: -12,
        signif: u256::new(
            0x7fffff555556eeeeea5ca5d895892a09,
            0xe70d6531485a5ea15620c615b21160f9,
        ),
    },
    // 2.44140620149361764016722943259659986212417790970617611807900460910178470217462e-4
    BigFloat {
        sign: 1,
        exp: -13,
        signif: u256::new(
            0x7fffffd555556eeeeedca5ca6adeaddf,
            0x3bc53a88bfc94608c52aa7843f7a3f03,
        ),
    },
    // 1.22070311893670204239058646117956300930829409015787498451939837846642590220456e-4
    BigFloat {
        sign: 1,
        exp: -14,
        signif: u256::new(
            0x7ffffff5555556eeeeeea5ca5cb40340,
            0x311a8606152723c4795ee346b7d5f4a7,
        ),
    },
    // 6.10351561742087750216625691738291537851435368333461793376711343165865657768902e-5
    BigFloat {
        sign: 1,
        exp: -15,
        signif: u256::new(
            0x7ffffffd5555556eeeeeedca5ca5d895,
            0x8957db5accfc793bf04dcaae26c692be,
        ),
    },
    // 3.05175781155260968618259534385360197509496751194378375310211568836116304861112e-5
    BigFloat {
        sign: 1,
        exp: -16,
        signif: u256::new(
            0x7fffffff55555556eeeeeeea5ca5ca6a,
            0xdeadeab02247f691462ba669daface7a,
        ),
    },
    // 1.52587890613157621072319358126978851374292381445758748462411864074458642670768e-5
    BigFloat {
        sign: 1,
        exp: -17,
        signif: u256::new(
            0x7fffffffd55555556eeeeeeedca5ca5c,
            0xb4034033f79d4b491b80fde149e9b5e1,
        ),
    },
    // 7.62939453110197026338848234010509058635074391846807715776383069653336854010974e-6
    BigFloat {
        sign: 1,
        exp: -18,
        signif: u256::new(
            0x7ffffffff555555556eeeeeeeea5ca5c,
            0xa5d895895892a09e66fe5336a7c7719d,
        ),
    },
    // 3.81469726560649628292307561637299372280525730396886631018743925039388846361042e-6
    BigFloat {
        sign: 1,
        exp: -19,
        signif: u256::new(
            0x7ffffffffd555555556eeeeeeeedca5c,
            0xa5ca6adeadeaddf3bc530b0bfd1ce914,
        ),
    },
    // 1.90734863281018703536536930591724416871434216545015336667005772346706446370985e-6
    BigFloat {
        sign: 1,
        exp: -20,
        signif: u256::new(
            0x7fffffffff5555555556eeeeeeeeea5c,
            0xa5ca5cb40340340311a860577a723e6a,
        ),
    },
    // 9.53674316405960879420670689923112390019634124498790160133611802076003329888111e-7
    BigFloat {
        sign: 1,
        exp: -21,
        signif: u256::new(
            0x7fffffffffd5555555556eeeeeeeeedc,
            0xa5ca5ca5d8958958957db5accf2a13bf,
        ),
    },
    // 4.76837158203088859927583821449247075870494043786641967400532158871423638144427e-7
    BigFloat {
        sign: 1,
        exp: -22,
        signif: u256::new(
            0x7ffffffffff55555555556eeeeeeeeee,
            0xa5ca5ca5ca6adeadeadeab02247f5f3c,
        ),
    },
    // 2.38418579101557982490947977218932697830968987690631559137669113722176482821030e-7
    BigFloat {
        sign: 1,
        exp: -23,
        signif: u256::new(
            0x7ffffffffffd55555555556eeeeeeeee,
            0xedca5ca5ca5cb4034034033f79d4b491,
        ),
    },
    // 1.19209289550780685311368497137922112645967587664586735576738225215437199588955e-7
    BigFloat {
        sign: 1,
        exp: -24,
        signif: u256::new(
            0x7fffffffffff555555555556eeeeeeee,
            0xeeea5ca5ca5ca5d895895895892a09e6,
        ),
    },
    // 5.96046447753905544139210621417888742500301957823662973142945657100051084616589e-8
    BigFloat {
        sign: 1,
        exp: -25,
        signif: u256::new(
            0x7fffffffffffd555555555556eeeeeee,
            0xeeeedca5ca5ca5ca6adeadeadeaddf3c,
        ),
    },
    // 2.98023223876953036767401327677095033490439070674451072492584778408435572608471e-8
    BigFloat {
        sign: 1,
        exp: -26,
        signif: u256::new(
            0x7ffffffffffff5555555555556eeeeee,
            0xeeeeeea5ca5ca5ca5cb4034034034031,
        ),
    },
    // 1.49011611938476551470925165959632471082489300259647200121700578054910142067273e-8
    BigFloat {
        sign: 1,
        exp: -27,
        signif: u256::new(
            0x7ffffffffffffd5555555555556eeeee,
            0xeeeeeeedca5ca5ca5ca5d89589589589,
        ),
    },
    // 7.45058059692382798713656457449539211320669255456658700759476014161737118369824e-9
    BigFloat {
        sign: 1,
        exp: -28,
        signif: u256::new(
            0x7fffffffffffff55555555555556eeee,
            0xeeeeeeeeea5ca5ca5ca5ca6adeadeadf,
        ),
    },
    // 3.72529029846191404526707057181192358367194832873704052423199826923910739743582e-9
    BigFloat {
        sign: 1,
        exp: -29,
        signif: u256::new(
            0x7fffffffffffffd55555555555556eee,
            0xeeeeeeeeeedca5ca5ca5ca5cb4034034,
        ),
    },
    // 1.86264514923095702909588382147649043450652828357388635134910501249513025944308e-9
    BigFloat {
        sign: 1,
        exp: -30,
        signif: u256::new(
            0x7ffffffffffffff555555555555556ee,
            0xeeeeeeeeeeeea5ca5ca5ca5ca5d89589,
        ),
    },
    // 9.31322574615478515355735477684561303892926496149290673943768542421974553295731e-10
    BigFloat {
        sign: 1,
        exp: -31,
        signif: u256::new(
            0x7ffffffffffffffd555555555555556e,
            0xeeeeeeeeeeeeedca5ca5ca5ca5ca6adf,
        ),
    },
    // 4.65661287307739257778841934710570162973478638915616174213234925544146496939157e-10
    BigFloat {
        sign: 1,
        exp: -32,
        signif: u256::new(
            0x7fffffffffffffff5555555555555556,
            0xeeeeeeeeeeeeeeea5ca5ca5ca5ca5cb4,
        ),
    },
    // 2.32830643653869628902042741838821270371274293204981860525486662280607146387632e-10
    BigFloat {
        sign: 1,
        exp: -33,
        signif: u256::new(
            0x7fffffffffffffffd555555555555555,
            0x6eeeeeeeeeeeeeeedca5ca5ca5ca5ca6,
        ),
    },
    // 1.16415321826934814452599092729852658796396457380014290026584979170884685731427e-10
    BigFloat {
        sign: 1,
        exp: -34,
        signif: u256::new(
            0x7ffffffffffffffff555555555555555,
            0x56eeeeeeeeeeeeeeeea5ca5ca5ca5ca6,
        ),
    },
    // 5.82076609134674072264967615912315823495491562577952724239762061671471623655998e-11
    BigFloat {
        sign: 1,
        exp: -35,
        signif: u256::new(
            0x7ffffffffffffffffd55555555555555,
            0x556eeeeeeeeeeeeeeeedca5ca5ca5ca6,
        ),
    },
    // 2.91038304567337036132730326989039477936936320036398304958299345250291480963432e-11
    BigFloat {
        sign: 1,
        exp: -36,
        signif: u256::new(
            0x7fffffffffffffffff55555555555555,
            0x5556eeeeeeeeeeeeeeeeea5ca5ca5ca6,
        ),
    },
    // 1.45519152283668518066395978373629934742117036089367107320672702133070941642533e-11
    BigFloat {
        sign: 1,
        exp: -37,
        signif: u256::new(
            0x7fffffffffffffffffd5555555555555,
            0x55556eeeeeeeeeeeeeeeeedca5ca5ca6,
        ),
    },
    // 7.27595761418342590332018410467037418427646293888214296401117528908389857511006e-12
    BigFloat {
        sign: 1,
        exp: -38,
        signif: u256::new(
            0x7ffffffffffffffffff5555555555555,
            0x555556eeeeeeeeeeeeeeeeeea5ca5ca6,
        ),
    },
    // 3.63797880709171295166014020058379677303455786697792581182960836464857409876385e-12
    BigFloat {
        sign: 1,
        exp: -39,
        signif: u256::new(
            0x7ffffffffffffffffffd555555555555,
            0x5555556eeeeeeeeeeeeeeeeeedca5ca6,
        ),
    },
    // 1.81898940354585647583007611882297459662931973336029253714520765350335530055238e-12
    BigFloat {
        sign: 1,
        exp: -40,
        signif: u256::new(
            0x7fffffffffffffffffff555555555555,
            0x55555556eeeeeeeeeeeeeeeeeeea5ca6,
        ),
    },
    // 9.09494701772928237915038811727871824578664966669663186226479288185490769828871e-13
    BigFloat {
        sign: 1,
        exp: -41,
        signif: u256::new(
            0x7fffffffffffffffffffd55555555555,
            0x555555556eeeeeeeeeeeeeeeeeeedca6,
        ),
    },
    // 4.54747350886464118957519499903483978072333120833696230124663921382485451117127e-13
    BigFloat {
        sign: 1,
        exp: -42,
        signif: u256::new(
            0x7ffffffffffffffffffff55555555555,
            0x5555555556eeeeeeeeeeeeeeeeeeeea6,
        ),
    },
    // 2.27373675443232059478759761706685497259041640104211664135781552996538778417206e-13
    BigFloat {
        sign: 1,
        exp: -43,
        signif: u256::new(
            0x7ffffffffffffffffffffd5555555555,
            0x55555555556eeeeeeeeeeeeeeeeeeeee,
        ),
    },
    // 1.13686837721616029739379882322710687157380205013026446622291399212808850334261e-13
    BigFloat {
        sign: 1,
        exp: -44,
        signif: u256::new(
            0x7fffffffffffffffffffff5555555555,
            0x555555555556eeeeeeeeeeeeeeeeeeef,
        ),
    },
    // 5.68434188608080148696899413450263358946725256266283054717026344356086532615360e-14
    BigFloat {
        sign: 1,
        exp: -45,
        signif: u256::new(
            0x7fffffffffffffffffffffd555555555,
            0x5555555555556eeeeeeeeeeeeeeeeeef,
        ),
    },
    // 2.84217094304040074348449706954720419868340657033285381728352108523888175004968e-14
    BigFloat {
        sign: 1,
        exp: -46,
        signif: u256::new(
            0x7ffffffffffffffffffffff555555555,
            0x55555555555556eeeeeeeeeeeeeeeeef,
        ),
    },
    // 1.42108547152020037174224853506058802483542582129160672712566632799216564326498e-14
    BigFloat {
        sign: 1,
        exp: -47,
        signif: u256::new(
            0x7ffffffffffffffffffffffd55555555,
            0x555555555555556eeeeeeeeeeeeeeeef,
        ),
    },
    // 7.10542735760100185871124267566167253104428227661450840889621609509561499924022e-15
    BigFloat {
        sign: 1,
        exp: -48,
        signif: u256::new(
            0x7fffffffffffffffffffffff55555555,
            0x5555555555555556eeeeeeeeeeeeeeef,
        ),
    },
    // 3.55271367880050092935562133787567781638053528457681355111168742392149587319125e-15
    BigFloat {
        sign: 1,
        exp: -49,
        signif: u256::new(
            0x7fffffffffffffffffffffffd5555555,
            0x55555555555555556eeeeeeeeeeeeeef,
        ),
    },
    // 1.77635683940025046467781066894344410204756691057210169388895031586626648409535e-15
    BigFloat {
        sign: 1,
        exp: -50,
        signif: u256::new(
            0x7ffffffffffffffffffffffff5555555,
            0x555555555555555556eeeeeeeeeeeeef,
        ),
    },
    // 8.88178419700125232338905334472422700255945863821512711736118457854410794885246e-16
    BigFloat {
        sign: 1,
        exp: -51,
        signif: u256::new(
            0x7ffffffffffffffffffffffffd555555,
            0x5555555555555555556eeeeeeeeeeeef,
        ),
    },
    // 4.44089209850062616169452667236298931281993232977689088967014796868399083247323e-16
    BigFloat {
        sign: 1,
        exp: -52,
        signif: u256::new(
            0x7fffffffffffffffffffffffff555555,
            0x55555555555555555556eeeeeeeeeeef,
        ),
    },
    // 2.22044604925031308084726333618160413285249154122211136120876849284693564589874e-16
    BigFloat {
        sign: 1,
        exp: -53,
        signif: u256::new(
            0x7fffffffffffffffffffffffffd55555,
            0x555555555555555555556eeeeeeeeeef,
        ),
    },
    // 1.11022302462515654042363166809081575098156144265276392015109606150466185548233e-16
    BigFloat {
        sign: 1,
        exp: -54,
        signif: u256::new(
            0x7ffffffffffffffffffffffffff55555,
            0x5555555555555555555556eeeeeeeeef,
        ),
    },
    // 5.55111512312578270211815834045409586060195180331595490018887007684920072552323e-17
    BigFloat {
        sign: 1,
        exp: -55,
        signif: u256::new(
            0x7ffffffffffffffffffffffffffd5555,
            0x55555555555555555555556eeeeeeeef,
        ),
    },
    // 2.77555756156289135105907917022705006851274397541449436252360875960516175963323e-17
    BigFloat {
        sign: 1,
        exp: -56,
        signif: u256::new(
            0x7fffffffffffffffffffffffffff5555,
            0x555555555555555555555556eeeeeeef,
        ),
    },
    // 1.38777878078144567552953958511352530153284299692681179531545109495061433460862e-17
    BigFloat {
        sign: 1,
        exp: -57,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffd555,
            0x5555555555555555555555556eeeeeef,
        ),
    },
    // 6.93889390390722837764769792556762684175980374615851474414431386868825826659030e-18
    BigFloat {
        sign: 1,
        exp: -58,
        signif: u256::new(
            0x7ffffffffffffffffffffffffffff555,
            0x55555555555555555555555556eeeeef,
        ),
    },
    // 3.46944695195361418882384896278381346264185046826981434301803923358603198170909e-18
    BigFloat {
        sign: 1,
        exp: -59,
        signif: u256::new(
            0x7ffffffffffffffffffffffffffffd55,
            0x555555555555555555555555556eeeef,
        ),
    },
    // 1.73472347597680709441192448139190673654116880853372679287725490419825398828818e-18
    BigFloat {
        sign: 1,
        exp: -60,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffff55,
            0x5555555555555555555555555556eeef,
        ),
    },
    // 8.67361737988403547205962240695953368923114851066715849109656863024781748241477e-19
    BigFloat {
        sign: 1,
        exp: -61,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffd5,
            0x55555555555555555555555555556eef,
        ),
    },
    // 4.33680868994201773602981120347976684543123731383339481138707107878097718520980e-19
    BigFloat {
        sign: 1,
        exp: -62,
        signif: u256::new(
            0x7ffffffffffffffffffffffffffffff5,
            0x555555555555555555555555555556ef,
        ),
    },
    // 2.16840434497100886801490560173988342281757653922917435142338388484762214814835e-19
    BigFloat {
        sign: 1,
        exp: -63,
        signif: u256::new(
            0x7ffffffffffffffffffffffffffffffd,
            0x5555555555555555555555555555556f,
        ),
    },
    // 1.08420217248550443400745280086994171142153300490364679392792298560595276851845e-19
    BigFloat {
        sign: 1,
        exp: -64,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0x55555555555555555555555555555557,
        ),
    },
    // 5.42101086242752217003726400434970855712359594362955849240990373200744096064800e-20
    BigFloat {
        sign: 1,
        exp: -65,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xd5555555555555555555555555555555,
        ),
    },
    // 2.71050543121376108501863200217485427856378933670369481155123796650093012008099e-20
    BigFloat {
        sign: 1,
        exp: -66,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xf5555555555555555555555555555555,
        ),
    },
    // 1.35525271560688054250931600108742713928214358896296185144390474581261626501012e-20
    BigFloat {
        sign: 1,
        exp: -67,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfd555555555555555555555555555555,
        ),
    },
    // 6.77626357803440271254658000543713569641102909557870231430488093226577033126262e-21
    BigFloat {
        sign: 1,
        exp: -68,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xff555555555555555555555555555555,
        ),
    },
    // 3.38813178901720135627329000271856784820555344163483778928811011653322129140781e-21
    BigFloat {
        sign: 1,
        exp: -69,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffd55555555555555555555555555555,
        ),
    },
    // 1.69406589450860067813664500135928392410278158254810472366101376456665266142597e-21
    BigFloat {
        sign: 1,
        exp: -70,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfff55555555555555555555555555555,
        ),
    },
    // 8.47032947254300339068322500679641962051391398990388090457626720570831582678242e-22
    BigFloat {
        sign: 1,
        exp: -71,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffd5555555555555555555555555555,
        ),
    },
    // 4.23516473627150169534161250339820981025695775459736011307203340071353947834778e-22
    BigFloat {
        sign: 1,
        exp: -72,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffff5555555555555555555555555555,
        ),
    },
    // 2.11758236813575084767080625169910490512847897225435751413400417508919243479346e-22
    BigFloat {
        sign: 1,
        exp: -73,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffd555555555555555555555555555,
        ),
    },
    // 1.05879118406787542383540312584955245256423949799663843926675052188614905434918e-22
    BigFloat {
        sign: 1,
        exp: -74,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffff555555555555555555555555555,
        ),
    },
    // 5.29395592033937711917701562924776226282119750482001679908343815235768631793645e-23
    BigFloat {
        sign: 1,
        exp: -75,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffd55555555555555555555555555,
        ),
    },
    // 2.64697796016968855958850781462388113141059875426461147488542976904471078974204e-23
    BigFloat {
        sign: 1,
        exp: -76,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffff55555555555555555555555555,
        ),
    },
    // 1.32348898008484427979425390731194056570529937736413112186067872113058884871775e-23
    BigFloat {
        sign: 1,
        exp: -77,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffd5555555555555555555555555,
        ),
    },
    // 6.61744490042422139897126953655970282852649688711043733982584840141323606089716e-24
    BigFloat {
        sign: 1,
        exp: -78,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffff5555555555555555555555555,
        ),
    },
    // 3.30872245021211069948563476827985141426324844359144138622823105017665450761213e-24
    BigFloat {
        sign: 1,
        exp: -79,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffd555555555555555555555555,
        ),
    },
    // 1.65436122510605534974281738413992570713162422180024853265352888127208181345151e-24
    BigFloat {
        sign: 1,
        exp: -80,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffff555555555555555555555555,
        ),
    },
    // 8.27180612553027674871408692069962853565812110900690246269191110159010226681435e-25
    BigFloat {
        sign: 1,
        exp: -81,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffd55555555555555555555555,
        ),
    },
    // 4.13590306276513837435704346034981426782906055450415870627398888769876278335178e-25
    BigFloat {
        sign: 1,
        exp: -82,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffff55555555555555555555555,
        ),
    },
    // 2.06795153138256918717852173017490713391453027725216778750299861096234534791896e-25
    BigFloat {
        sign: 1,
        exp: -83,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffd5555555555555555555555,
        ),
    },
    // 1.03397576569128459358926086508745356695726513862609494804724982637029316848987e-25
    BigFloat {
        sign: 1,
        exp: -84,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffff5555555555555555555555,
        ),
    },
    // 5.16987882845642296794630432543726783478632569313048855810593728296286646061231e-26
    BigFloat {
        sign: 1,
        exp: -85,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffd555555555555555555555,
        ),
    },
    // 2.58493941422821148397315216271863391739316284656524600628667966037035830757653e-26
    BigFloat {
        sign: 1,
        exp: -86,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffff555555555555555555555,
        ),
    },
    // 1.29246970711410574198657608135931695869658142328262321904755370754629478844706e-26
    BigFloat {
        sign: 1,
        exp: -87,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffd55555555555555555555,
        ),
    },
    // 6.46234853557052870993288040679658479348290711641311636511803588443286848555880e-27
    BigFloat {
        sign: 1,
        exp: -88,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffff55555555555555555555,
        ),
    },
    // 3.23117426778526435496644020339829239674145355820655821629405136055410856069484e-27
    BigFloat {
        sign: 1,
        exp: -89,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffd5555555555555555555,
        ),
    },
    // 1.61558713389263217748322010169914619837072677910327911236390485756926357008685e-27
    BigFloat {
        sign: 1,
        exp: -90,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffff5555555555555555555,
        ),
    },
    // 8.07793566946316088741610050849573099185363389551639556709062325946157946260852e-28
    BigFloat {
        sign: 1,
        exp: -91,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffd555555555555555555,
        ),
    },
    // 4.03896783473158044370805025424786549592681694775819778420419900118269743282605e-28
    BigFloat {
        sign: 1,
        exp: -92,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffff555555555555555555,
        ),
    },
    // 2.01948391736579022185402512712393274796340847387909889218446042202283717910325e-28
    BigFloat {
        sign: 1,
        exp: -93,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffd55555555555555555,
        ),
    },
    // 1.00974195868289511092701256356196637398170423693954944610252532619035464738790e-28
    BigFloat {
        sign: 1,
        exp: -94,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffff55555555555555555,
        ),
    },
    // 5.04870979341447555463506281780983186990852118469774723052549552492544330923486e-29
    BigFloat {
        sign: 1,
        exp: -95,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffd5555555555555555,
        ),
    },
    // 2.52435489670723777731753140890491593495426059234887361526435637420943041365435e-29
    BigFloat {
        sign: 1,
        exp: -96,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffff5555555555555555,
        ),
    },
    // 1.26217744835361888865876570445245796747713029617443680763237926357305380170679e-29
    BigFloat {
        sign: 1,
        exp: -97,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffd555555555555555,
        ),
    },
    // 6.31088724176809444329382852226228983738565148087218403816214766345069225213346e-30
    BigFloat {
        sign: 1,
        exp: -98,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffff555555555555555,
        ),
    },
    // 3.15544362088404722164691426113114491869282574043609201908110524992352403151667e-30
    BigFloat {
        sign: 1,
        exp: -99,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffd55555555555555,
        ),
    },
    // 1.57772181044202361082345713056557245934641287021804600954055655223653425393958e-30
    BigFloat {
        sign: 1,
        exp: -100,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffff55555555555555,
        ),
    },
    // 7.88860905221011805411728565282786229673206435109023004770278767027613656742444e-31
    BigFloat {
        sign: 1,
        exp: -101,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffd5555555555555,
        ),
    },
    // 3.94430452610505902705864282641393114836603217554511502385139444877475144592804e-31
    BigFloat {
        sign: 1,
        exp: -102,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffff5555555555555,
        ),
    },
    // 1.97215226305252951352932141320696557418301608777255751192569730109196111824100e-31
    BigFloat {
        sign: 1,
        exp: -103,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffd555555555555,
        ),
    },
    // 9.86076131526264756764660706603482787091508043886278755962848660134053733530120e-32
    BigFloat {
        sign: 1,
        exp: -104,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffff555555555555,
        ),
    },
    // 4.93038065763132378382330353301741393545754021943139377981424331265536013566263e-32
    BigFloat {
        sign: 1,
        exp: -105,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffd55555555555,
        ),
    },
    // 2.46519032881566189191165176650870696772877010971569688990712165782581650133282e-32
    BigFloat {
        sign: 1,
        exp: -106,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffff55555555555,
        ),
    },
    // 1.23259516440783094595582588325435348386438505485784844495356082910017530485410e-32
    BigFloat {
        sign: 1,
        exp: -107,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffd5555555555,
        ),
    },
    // 6.16297582203915472977912941627176741932192527428924222476780414573496034200509e-33
    BigFloat {
        sign: 1,
        exp: -108,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffff5555555555,
        ),
    },
    // 3.08148791101957736488956470813588370966096263714462111238390207289674064821937e-33
    BigFloat {
        sign: 1,
        exp: -109,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffd555555555,
        ),
    },
    // 1.54074395550978868244478235406794185483048131857231055619195103645202788376179e-33
    BigFloat {
        sign: 1,
        exp: -110,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffff555555555,
        ),
    },
    // 7.70371977754894341222391177033970927415240659286155278095975518226471136837408e-34
    BigFloat {
        sign: 1,
        exp: -111,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffd55555555,
        ),
    },
    // 3.85185988877447170611195588516985463707620329643077639047987759113292717788268e-34
    BigFloat {
        sign: 1,
        exp: -112,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffff55555555,
        ),
    },
    // 1.92592994438723585305597794258492731853810164821538819523993879556653502565330e-34
    BigFloat {
        sign: 1,
        exp: -113,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffd5555555,
        ),
    },
    // 9.62964972193617926527988971292463659269050824107694097619969397783276442415642e-35
    BigFloat {
        sign: 1,
        exp: -114,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffff5555555,
        ),
    },
    // 4.81482486096808963263994485646231829634525412053847048809984698891639337406445e-35
    BigFloat {
        sign: 1,
        exp: -115,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffffd555555,
        ),
    },
    // 2.40741243048404481631997242823115914817262706026923524404992349445819808228051e-35
    BigFloat {
        sign: 1,
        exp: -116,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffff555555,
        ),
    },
    // 1.20370621524202240815998621411557957408631353013461762202496174722909921554629e-35
    BigFloat {
        sign: 1,
        exp: -117,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffffd55555,
        ),
    },
    // 6.01853107621011204079993107057789787043156765067308811012480873614549629573899e-36
    BigFloat {
        sign: 1,
        exp: -118,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffffff55555,
        ),
    },
    // 3.00926553810505602039996553528894893521578382533654405506240436807274817512044e-36
    BigFloat {
        sign: 1,
        exp: -119,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffffffd5555,
        ),
    },
    // 1.50463276905252801019998276764447446760789191266827202753120218403637409096659e-36
    BigFloat {
        sign: 1,
        exp: -120,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffffff5555,
        ),
    },
    // 7.52316384526264005099991383822237233803945956334136013765601092018187045909089e-37
    BigFloat {
        sign: 1,
        exp: -121,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffffffd555,
        ),
    },
    // 3.76158192263132002549995691911118616901972978167068006882800546009093523007769e-37
    BigFloat {
        sign: 1,
        exp: -122,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffffffff555,
        ),
    },
    // 1.88079096131566001274997845955559308450986489083534003441400273004546761510538e-37
    BigFloat {
        sign: 1,
        exp: -123,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffffffffd55,
        ),
    },
    // 9.40395480657830006374989229777796542254932445417670017207001365022733807561004e-38
    BigFloat {
        sign: 1,
        exp: -124,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffffffff55,
        ),
    },
    // 4.70197740328915003187494614888898271127466222708835008603500682511366903781542e-38
    BigFloat {
        sign: 1,
        exp: -125,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffffffffd5,
        ),
    },
    // 2.35098870164457501593747307444449135563733111354417504301750341255683451890901e-38
    BigFloat {
        sign: 1,
        exp: -126,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffffffffff5,
        ),
    },
    // 1.17549435082228750796873653722224567781866555677208752150875170627841725945467e-38
    BigFloat {
        sign: 1,
        exp: -127,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xfffffffffffffffffffffffffffffffd,
        ),
    },
    // 5.87747175411143753984368268611122838909332778386043760754375853139208629727353e-39
    BigFloat {
        sign: 1,
        exp: -128,
        signif: u256::new(
            0x7fffffffffffffffffffffffffffffff,
            0xffffffffffffffffffffffffffffffff,
        ),
    },
    // 2.93873587705571876992184134305561419454666389193021880377187926569604314863682e-39
    BigFloat {
        sign: 1,
        exp: -128,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.46936793852785938496092067152780709727333194596510940188593963284802157431841e-39
    BigFloat {
        sign: 1,
        exp: -129,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.34683969263929692480460335763903548636665972982554700942969816424010787159204e-40
    BigFloat {
        sign: 1,
        exp: -130,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.67341984631964846240230167881951774318332986491277350471484908212005393579602e-40
    BigFloat {
        sign: 1,
        exp: -131,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.83670992315982423120115083940975887159166493245638675235742454106002696789801e-40
    BigFloat {
        sign: 1,
        exp: -132,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 9.18354961579912115600575419704879435795832466228193376178712270530013483949006e-41
    BigFloat {
        sign: 1,
        exp: -133,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.59177480789956057800287709852439717897916233114096688089356135265006741974503e-41
    BigFloat {
        sign: 1,
        exp: -134,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.29588740394978028900143854926219858948958116557048344044678067632503370987251e-41
    BigFloat {
        sign: 1,
        exp: -135,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.14794370197489014450071927463109929474479058278524172022339033816251685493626e-41
    BigFloat {
        sign: 1,
        exp: -136,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.73971850987445072250359637315549647372395291392620860111695169081258427468129e-42
    BigFloat {
        sign: 1,
        exp: -137,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.86985925493722536125179818657774823686197645696310430055847584540629213734064e-42
    BigFloat {
        sign: 1,
        exp: -138,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.43492962746861268062589909328887411843098822848155215027923792270314606867032e-42
    BigFloat {
        sign: 1,
        exp: -139,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.17464813734306340312949546644437059215494114240776075139618961351573034335161e-43
    BigFloat {
        sign: 1,
        exp: -140,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.58732406867153170156474773322218529607747057120388037569809480675786517167580e-43
    BigFloat {
        sign: 1,
        exp: -141,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.79366203433576585078237386661109264803873528560194018784904740337893258583790e-43
    BigFloat {
        sign: 1,
        exp: -142,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 8.96831017167882925391186933305546324019367642800970093924523701689466292918951e-44
    BigFloat {
        sign: 1,
        exp: -143,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.48415508583941462695593466652773162009683821400485046962261850844733146459475e-44
    BigFloat {
        sign: 1,
        exp: -144,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.24207754291970731347796733326386581004841910700242523481130925422366573229738e-44
    BigFloat {
        sign: 1,
        exp: -145,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.12103877145985365673898366663193290502420955350121261740565462711183286614869e-44
    BigFloat {
        sign: 1,
        exp: -146,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.60519385729926828369491833315966452512104776750606308702827313555916433074344e-45
    BigFloat {
        sign: 1,
        exp: -147,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.80259692864963414184745916657983226256052388375303154351413656777958216537172e-45
    BigFloat {
        sign: 1,
        exp: -148,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.40129846432481707092372958328991613128026194187651577175706828388979108268586e-45
    BigFloat {
        sign: 1,
        exp: -149,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.00649232162408535461864791644958065640130970938257885878534141944895541342930e-46
    BigFloat {
        sign: 1,
        exp: -150,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.50324616081204267730932395822479032820065485469128942939267070972447770671465e-46
    BigFloat {
        sign: 1,
        exp: -151,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.75162308040602133865466197911239516410032742734564471469633535486223885335733e-46
    BigFloat {
        sign: 1,
        exp: -152,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 8.75811540203010669327330989556197582050163713672822357348167677431119426678663e-47
    BigFloat {
        sign: 1,
        exp: -153,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.37905770101505334663665494778098791025081856836411178674083838715559713339331e-47
    BigFloat {
        sign: 1,
        exp: -154,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.18952885050752667331832747389049395512540928418205589337041919357779856669666e-47
    BigFloat {
        sign: 1,
        exp: -155,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.09476442525376333665916373694524697756270464209102794668520959678889928334833e-47
    BigFloat {
        sign: 1,
        exp: -156,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.47382212626881668329581868472623488781352321045513973342604798394449641674164e-48
    BigFloat {
        sign: 1,
        exp: -157,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.73691106313440834164790934236311744390676160522756986671302399197224820837082e-48
    BigFloat {
        sign: 1,
        exp: -158,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.36845553156720417082395467118155872195338080261378493335651199598612410418541e-48
    BigFloat {
        sign: 1,
        exp: -159,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 6.84227765783602085411977335590779360976690401306892466678255997993062052092705e-49
    BigFloat {
        sign: 1,
        exp: -160,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.42113882891801042705988667795389680488345200653446233339127998996531026046353e-49
    BigFloat {
        sign: 1,
        exp: -161,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.71056941445900521352994333897694840244172600326723116669563999498265513023176e-49
    BigFloat {
        sign: 1,
        exp: -162,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 8.55284707229502606764971669488474201220863001633615583347819997491327565115882e-50
    BigFloat {
        sign: 1,
        exp: -163,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.27642353614751303382485834744237100610431500816807791673909998745663782557941e-50
    BigFloat {
        sign: 1,
        exp: -164,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.13821176807375651691242917372118550305215750408403895836954999372831891278970e-50
    BigFloat {
        sign: 1,
        exp: -165,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.06910588403687825845621458686059275152607875204201947918477499686415945639485e-50
    BigFloat {
        sign: 1,
        exp: -166,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.34552942018439129228107293430296375763039376021009739592387498432079728197426e-51
    BigFloat {
        sign: 1,
        exp: -167,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.67276471009219564614053646715148187881519688010504869796193749216039864098713e-51
    BigFloat {
        sign: 1,
        exp: -168,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.33638235504609782307026823357574093940759844005252434898096874608019932049357e-51
    BigFloat {
        sign: 1,
        exp: -169,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 6.68191177523048911535134116787870469703799220026262174490484373040099660246783e-52
    BigFloat {
        sign: 1,
        exp: -170,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.34095588761524455767567058393935234851899610013131087245242186520049830123391e-52
    BigFloat {
        sign: 1,
        exp: -171,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.67047794380762227883783529196967617425949805006565543622621093260024915061696e-52
    BigFloat {
        sign: 1,
        exp: -172,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 8.35238971903811139418917645984838087129749025032827718113105466300124575308478e-53
    BigFloat {
        sign: 1,
        exp: -173,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.17619485951905569709458822992419043564874512516413859056552733150062287654239e-53
    BigFloat {
        sign: 1,
        exp: -174,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.08809742975952784854729411496209521782437256258206929528276366575031143827120e-53
    BigFloat {
        sign: 1,
        exp: -175,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.04404871487976392427364705748104760891218628129103464764138183287515571913560e-53
    BigFloat {
        sign: 1,
        exp: -176,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.22024357439881962136823528740523804456093140645517323820690916437577859567799e-54
    BigFloat {
        sign: 1,
        exp: -177,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.61012178719940981068411764370261902228046570322758661910345458218788929783899e-54
    BigFloat {
        sign: 1,
        exp: -178,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.30506089359970490534205882185130951114023285161379330955172729109394464891950e-54
    BigFloat {
        sign: 1,
        exp: -179,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 6.52530446799852452671029410925654755570116425806896654775863645546972324459749e-55
    BigFloat {
        sign: 1,
        exp: -180,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.26265223399926226335514705462827377785058212903448327387931822773486162229874e-55
    BigFloat {
        sign: 1,
        exp: -181,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.63132611699963113167757352731413688892529106451724163693965911386743081114937e-55
    BigFloat {
        sign: 1,
        exp: -182,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 8.15663058499815565838786763657068444462645532258620818469829556933715405574686e-56
    BigFloat {
        sign: 1,
        exp: -183,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.07831529249907782919393381828534222231322766129310409234914778466857702787343e-56
    BigFloat {
        sign: 1,
        exp: -184,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.03915764624953891459696690914267111115661383064655204617457389233428851393671e-56
    BigFloat {
        sign: 1,
        exp: -185,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.01957882312476945729848345457133555557830691532327602308728694616714425696836e-56
    BigFloat {
        sign: 1,
        exp: -186,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.09789411562384728649241727285667777789153457661638011543643473083572128484179e-57
    BigFloat {
        sign: 1,
        exp: -187,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.54894705781192364324620863642833888894576728830819005771821736541786064242089e-57
    BigFloat {
        sign: 1,
        exp: -188,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.27447352890596182162310431821416944447288364415409502885910868270893032121045e-57
    BigFloat {
        sign: 1,
        exp: -189,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 6.37236764452980910811552159107084722236441822077047514429554341354465160605223e-58
    BigFloat {
        sign: 1,
        exp: -190,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.18618382226490455405776079553542361118220911038523757214777170677232580302612e-58
    BigFloat {
        sign: 1,
        exp: -191,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.59309191113245227702888039776771180559110455519261878607388585338616290151306e-58
    BigFloat {
        sign: 1,
        exp: -192,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.96545955566226138514440198883855902795552277596309393036942926693081450756529e-59
    BigFloat {
        sign: 1,
        exp: -193,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.98272977783113069257220099441927951397776138798154696518471463346540725378265e-59
    BigFloat {
        sign: 1,
        exp: -194,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.99136488891556534628610049720963975698888069399077348259235731673270362689132e-59
    BigFloat {
        sign: 1,
        exp: -195,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 9.95682444457782673143050248604819878494440346995386741296178658366351813445661e-60
    BigFloat {
        sign: 1,
        exp: -196,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.97841222228891336571525124302409939247220173497693370648089329183175906722831e-60
    BigFloat {
        sign: 1,
        exp: -197,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.48920611114445668285762562151204969623610086748846685324044664591587953361415e-60
    BigFloat {
        sign: 1,
        exp: -198,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.24460305557222834142881281075602484811805043374423342662022332295793976680708e-60
    BigFloat {
        sign: 1,
        exp: -199,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 6.22301527786114170714406405378012424059025216872116713310111661478969883403538e-61
    BigFloat {
        sign: 1,
        exp: -200,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.11150763893057085357203202689006212029512608436058356655055830739484941701769e-61
    BigFloat {
        sign: 1,
        exp: -201,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.55575381946528542678601601344503106014756304218029178327527915369742470850885e-61
    BigFloat {
        sign: 1,
        exp: -202,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.77876909732642713393008006722515530073781521090145891637639576848712354254423e-62
    BigFloat {
        sign: 1,
        exp: -203,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.88938454866321356696504003361257765036890760545072945818819788424356177127211e-62
    BigFloat {
        sign: 1,
        exp: -204,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.94469227433160678348252001680628882518445380272536472909409894212178088563606e-62
    BigFloat {
        sign: 1,
        exp: -205,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 9.72346137165803391741260008403144412592226901362682364547049471060890442818029e-63
    BigFloat {
        sign: 1,
        exp: -206,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.86173068582901695870630004201572206296113450681341182273524735530445221409014e-63
    BigFloat {
        sign: 1,
        exp: -207,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.43086534291450847935315002100786103148056725340670591136762367765222610704507e-63
    BigFloat {
        sign: 1,
        exp: -208,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.21543267145725423967657501050393051574028362670335295568381183882611305352254e-63
    BigFloat {
        sign: 1,
        exp: -209,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 6.07716335728627119838287505251965257870141813351676477841905919413056526761268e-64
    BigFloat {
        sign: 1,
        exp: -210,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.03858167864313559919143752625982628935070906675838238920952959706528263380634e-64
    BigFloat {
        sign: 1,
        exp: -211,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.51929083932156779959571876312991314467535453337919119460476479853264131690317e-64
    BigFloat {
        sign: 1,
        exp: -212,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.59645419660783899797859381564956572337677266689595597302382399266320658451585e-65
    BigFloat {
        sign: 1,
        exp: -213,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.79822709830391949898929690782478286168838633344797798651191199633160329225792e-65
    BigFloat {
        sign: 1,
        exp: -214,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.89911354915195974949464845391239143084419316672398899325595599816580164612896e-65
    BigFloat {
        sign: 1,
        exp: -215,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 9.49556774575979874747324226956195715422096583361994496627977999082900823064481e-66
    BigFloat {
        sign: 1,
        exp: -216,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.74778387287989937373662113478097857711048291680997248313988999541450411532241e-66
    BigFloat {
        sign: 1,
        exp: -217,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.37389193643994968686831056739048928855524145840498624156994499770725205766120e-66
    BigFloat {
        sign: 1,
        exp: -218,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.18694596821997484343415528369524464427762072920249312078497249885362602883060e-66
    BigFloat {
        sign: 1,
        exp: -219,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.93472984109987421717077641847622322138810364601246560392486249426813014415301e-67
    BigFloat {
        sign: 1,
        exp: -220,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.96736492054993710858538820923811161069405182300623280196243124713406507207650e-67
    BigFloat {
        sign: 1,
        exp: -221,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.48368246027496855429269410461905580534702591150311640098121562356703253603825e-67
    BigFloat {
        sign: 1,
        exp: -222,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.41841230137484277146347052309527902673512955751558200490607811783516268019126e-68
    BigFloat {
        sign: 1,
        exp: -223,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.70920615068742138573173526154763951336756477875779100245303905891758134009563e-68
    BigFloat {
        sign: 1,
        exp: -224,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.85460307534371069286586763077381975668378238937889550122651952945879067004781e-68
    BigFloat {
        sign: 1,
        exp: -225,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 9.27301537671855346432933815386909878341891194689447750613259764729395335023907e-69
    BigFloat {
        sign: 1,
        exp: -226,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.63650768835927673216466907693454939170945597344723875306629882364697667511954e-69
    BigFloat {
        sign: 1,
        exp: -227,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.31825384417963836608233453846727469585472798672361937653314941182348833755977e-69
    BigFloat {
        sign: 1,
        exp: -228,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.15912692208981918304116726923363734792736399336180968826657470591174416877988e-69
    BigFloat {
        sign: 1,
        exp: -229,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.79563461044909591520583634616818673963681996680904844133287352955872084389942e-70
    BigFloat {
        sign: 1,
        exp: -230,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.89781730522454795760291817308409336981840998340452422066643676477936042194971e-70
    BigFloat {
        sign: 1,
        exp: -231,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.44890865261227397880145908654204668490920499170226211033321838238968021097486e-70
    BigFloat {
        sign: 1,
        exp: -232,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.24454326306136989400729543271023342454602495851131055166609191194840105487428e-71
    BigFloat {
        sign: 1,
        exp: -233,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.62227163153068494700364771635511671227301247925565527583304595597420052743714e-71
    BigFloat {
        sign: 1,
        exp: -234,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.81113581576534247350182385817755835613650623962782763791652297798710026371857e-71
    BigFloat {
        sign: 1,
        exp: -235,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 9.05567907882671236750911929088779178068253119813913818958261488993550131859285e-72
    BigFloat {
        sign: 1,
        exp: -236,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.52783953941335618375455964544389589034126559906956909479130744496775065929642e-72
    BigFloat {
        sign: 1,
        exp: -237,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.26391976970667809187727982272194794517063279953478454739565372248387532964821e-72
    BigFloat {
        sign: 1,
        exp: -238,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.13195988485333904593863991136097397258531639976739227369782686124193766482411e-72
    BigFloat {
        sign: 1,
        exp: -239,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.65979942426669522969319955680486986292658199883696136848913430620968832412053e-73
    BigFloat {
        sign: 1,
        exp: -240,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.82989971213334761484659977840243493146329099941848068424456715310484416206026e-73
    BigFloat {
        sign: 1,
        exp: -241,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.41494985606667380742329988920121746573164549970924034212228357655242208103013e-73
    BigFloat {
        sign: 1,
        exp: -242,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 7.07474928033336903711649944600608732865822749854620171061141788276211040515066e-74
    BigFloat {
        sign: 1,
        exp: -243,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.53737464016668451855824972300304366432911374927310085530570894138105520257533e-74
    BigFloat {
        sign: 1,
        exp: -244,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.76868732008334225927912486150152183216455687463655042765285447069052760128767e-74
    BigFloat {
        sign: 1,
        exp: -245,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 8.84343660041671129639562430750760916082278437318275213826427235345263800643833e-75
    BigFloat {
        sign: 1,
        exp: -246,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 4.42171830020835564819781215375380458041139218659137606913213617672631900321916e-75
    BigFloat {
        sign: 1,
        exp: -247,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.21085915010417782409890607687690229020569609329568803456606808836315950160958e-75
    BigFloat {
        sign: 1,
        exp: -248,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.10542957505208891204945303843845114510284804664784401728303404418157975080479e-75
    BigFloat {
        sign: 1,
        exp: -249,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 5.52714787526044456024726519219225572551424023323922008641517022090789875402395e-76
    BigFloat {
        sign: 1,
        exp: -250,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 2.76357393763022228012363259609612786275712011661961004320758511045394937701198e-76
    BigFloat {
        sign: 1,
        exp: -251,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 1.38178696881511114006181629804806393137856005830980502160379255522697468850599e-76
    BigFloat {
        sign: 1,
        exp: -252,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 6.90893484407555570030908149024031965689280029154902510801896277613487344252994e-77
    BigFloat {
        sign: 1,
        exp: -253,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
    // 3.45446742203777785015454074512015982844640014577451255400948138806743672126497e-77
    BigFloat {
        sign: 1,
        exp: -254,
        signif: u256::new(
            0x40000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    },
];

#[cfg(test)]
mod atan_table_tests {
    use super::*;
    use crate::{consts::FRAC_PI_4, f256};

    #[test]
    fn test_first_entry() {
        assert_eq!(f256::from(&ATANS[0]), FRAC_PI_4);
    }

    #[test]
    fn test_129th_entry() {
        assert_eq!(
            ATANS[128],
            BigFloat {
                sign: 1,
                exp: -128,
                signif: u256::new(
                    0x40000000000000000000000000000000,
                    0x00000000000000000000000000000000,
                ),
            },
        );
    }
}
