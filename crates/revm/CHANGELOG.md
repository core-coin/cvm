# v3.0.0
date 29.01.2022

This is big release that has core changes that breaks compatibility. In summary:
*  Project is refactored into `revm-primitives`,`revm-precompile`,`revm-interpreter` and `revm` to have more flexibility and separation of concerns. And include paths in revm reflect that. So try to find include as `revm::primitives` or `revm::interpreter`
* Parity `primitive-types` was replaced with `ruint` for big numbers and subset of macros are used for native `B176`/`B256` types. 
* Interpreter instructions are unified and now all of them have same signature.
* web3 db was replaces with ethers alternative.
* revmjs lib was removed from crates.
* `revm_precompiles` was renamed to `revm-precompile.`

* Return types are made to have more insight of what have happened inside revm.
* Snailtracer benchmark got around 20% faster.

Github Changelog:
* dc9818f - (HEAD -> o/bump, origin/bump_v20) Bump v20 (13 hours ago) <rakita>
* 75ef0f1 - (origin/main, origin/HEAD) feat: Staticcall internal return (#349) (13 hours ago) <rakita>
* 0194b37 - (t) fix bug introduced in last commit (13 hours ago) <rakita>
* 7b00f32 - Cleanup imports (#348) (14 hours ago) <rakita>
* c14d7ea - fix: enable the examples to run with the current revm (#347) (16 hours ago) <flyq>
* 329fd94 - Wrap all calls to interpreter.energy.erase_cost with checks if USE_energy is enabled (#346) (2 days ago) <christn>
* 72355f4 - improvement: add logs & return value to revert (#343) (3 days ago) <Wodann>
* 142a1c9 - expose hashbrown::HashMap in primitives (#345) (3 days ago) <Andy Thomson>
* ba393d7 - fix: disable balance check (#342) (4 days ago) <Wodann>
* 876fad1 - refactor: simplify DatabaseComponentError (#339) (6 days ago) <Wodann>
* 81534ad - chore: includes to libs (#338) (7 days ago) <rakita>
* e2f4d32 - Creating revm-primitives, revm better errors and db components  (#334) (10 days ago) <rakita>
* de83db6 - fix: feature flags (#330) (2 weeks ago) <Wodann>
* b60269c - `revm`: mark `with-serde` feature as deprecated (#328) (2 weeks ago) <Enrique Ortiz>
* 63bf475 - make load_account pub (#325) (3 weeks ago) <rakita>
* 0ef0197 - Cleanup, move hot fields toggether in Interpreter (#321) (3 weeks ago) <rakita>
* 81942d6 - enable proptest with arbitrary feature (#323) (3 weeks ago) <joshieDo>
* 2be3798 - feat: revm-interpreter created (#320) (3 weeks ago) <rakita>
* 7e98fef - fix: feature flag compiler errors (#256) (5 weeks ago) <Wodann>
* 488ef8a - Add example for fork + ref_transact impl (#296) (6 weeks ago) <0xDmtri>
* 56e6c22 - feat: allow disabling of balance checks (#297) (6 weeks ago) <Wodann>
* 8661467 - feat: Export CustomPrinter insector from revm (#300) (6 weeks ago) <rakita>
* 222b8e9 - feature: substitute web3db to ethersdb (#293) (6 weeks ago) <0xDmtri>
* fd01083 - feature(revm): Return `bytes` in Create calls (#289) (7 weeks ago) <Nicolas Gotchac>
* 2fb0933 - docs: Correct typo (#282) (7 weeks ago) <Przemyslaw Rzad>
* 90fe01e - feat(interpreter): Unify instruction fn signature (#283) (7 weeks ago) <rakita>
* 54e0333 - bug: Integer overflow while calculating the remaining energy in energyInspector (#287) (8 weeks ago) <rakita>
* acdbaac - native bits (#278) (8 weeks ago) <rakita>
* 69e302b - feat(revm): Add prevrandao field to EnvBlock (#271) (2 months ago) <rakita>
* d1703cd - Export StorageSlot (#265) (3 months ago) <Francesco Cinà>
* 560bb03 - Fix: typos (#263) (3 months ago) <HAPPY>
* 369244e - feat(refactor): make keccak in one place. (#247) (3 months ago) <rakita>
* c96c878 - feat: Migrate `primitive_types::U256` to `ruint::Uint<256, 4>` (#239) (3 months ago) <Alexey Shekhirin>


# v2.3.1
date: 22.11.2022

Bump dependency versions.


# v2.3.0
date: 16.11.2022
Very small release. Exposes one field and added prevrandao to remove footgun of forgeting to set difficulty.

* 927d16c - disable energy refunds with env flag (#267) (14 minutes ago) <gd>
* 47a8310 - Add prevrandao field to EnvBlock (3 minutes ago) <rakita>
* 2c45b04 - Export StorageSlot (#265) (23 minutes ago) <Francesco Cinà>

# v2.2.0
date: 12.11.2022

Small release that contains consensus bug fix. Additionaly added few small feature flags needed for hardhat, opcode utility function and removal of web3db block number check. 

* dc3414a - Added OEF spec for tests. Skip HighenergyPrice (4 minutes ago) <rakita>
* f462f9d - Bugfix: if returndatacopy is len 0 return after initial cost (#259) (4 minutes ago) <gd>
* ea2f2a2 - fix web3db sanity check (#245) (12 days ago) <Wulder>
* 9f8cdbd - feat: allow block energy limit to be toggled off (#238) (3 weeks ago) <Wodann>
* efd9afc - feat: allow eip3607 to be toggled off (#237) (3 weeks ago) <Wodann>
* 88c72a7 - fix: return out of energy code for precompiled contracts (#234) (3 weeks ago) <Wodann>
* 30462a3 - Fix: typos (#232) (3 weeks ago) <omahs>
* 9f513c1 - Borrow self and add derive traits for OpCode (#231) (4 weeks ago) <Franfran>

# v2.1.0
date: 25.09.2022

energyInspector added by Alexey Shekhirin and some helper functions.
Changes:

* ca14d61 - energy inspector (#222) (7 days ago) <Alexey Shekhirin>
* 1e25c99 - chore: expose original value on storageslot (#216) (13 days ago) <Matthias Seitz>
* aa39d64 - feat: add Memory::shrink_to_fit (#215) (13 days ago) <Matthias Seitz

# v2.0.0
date: 10.09.2022

Release with `Database` interface changed, execution result, consensus bug fixes and support for all past forks. Additional optimizations on evm initialization.

Main changes:
* Add support for old forks. (#191) (9 days ago)
* revm/evm: Return `ExecutionResult`, which includes `energy_refunded` (#169) (4 weeks ago) <Nicolas Gotchac>
* JournaledState (#175)
    * Optimize handling of precompiles. Initialization and account loading.
    * Fixes SELFDESTRUCT bug.
* Optimize calldataload. Some cleanup (#168)
* Handle HighNonce tests (#176)
* feat: expose hash on `BytecodeLocked` (#189) (12 days ago) <Bjerg>
* revm: Update account storage methods in CacheDB (#171) (4 weeks ago) <Nicolas Gotchac>
* reexport revm_precompiles as precompiles (#197) (6 days ago) <Matthias Seitz>
* chore(ci): use ethtests profile for CI tests (#188) (2 weeks ago) <Alexey Shekhirin>
* Bump dependencies version
* current_opcode fn and rename program_counter to instruction_pointer (#211)
* Cfg choose create analysis, option on bytecode size limit (#210)
* Cleanup remove U256 and use u64 for energy calculation (#213)

Consensus bugs:
* SELFDESTRUCT was not handled correctly. It would remove account/storage but it should just mark it for removal. This bug was here from earlier version of revm. (#175)
* fix: set energy_block to empty bytecode (#172). Introduced in v1.8.0 with bytecode format.

# v1.9.0
date: 09.08.2022

Small release. Optimizations

* Cache bytecode hash
* Move override_spec config from Inspector to cfg

# v1.8.0
date: 01.08.2022

Medium release, good performance boost. Database trait has changed to support Bytecode.

* Introduce Bytecode format (#156)
* Update readme files.
* Merge eth/tests supported.

# v1.7.0
date: 11.06.2022

small release:
* Make CacheDB field pub and add few utility functions
* Rename Byzantine to Byzantium

# v1.6.0
date: 02.06.2022

Most changes are relayed to CacheDB and how it saved accounts.

* Introduce account `Touched/Cleared/None` state in CacheDB
* Add missing inspectors `call_end` calls
* bump dependencies and few standard derives.

# v1.5.0
date: 09.06.2022

Consensus error related to energy block optimization and `sstore` min stipend. Solution is to make `sstore` instruction as `energy_block_end` as to not spend future instruction energy when checking min stipend condition introduced in EIP-2200.

* Consensus error with energy block for SSTORE stipend check (#124)
* enable EIP2200 in Istanbul (#125)

# v1.4.1
date: 06.06.2022

Small release:
* chore: export evm_inner (#122)

# v1.4.0
date: 03.06.2022

Small release:
* fix: BLOCKHASH should return 0 if number not in last 256 blocks (#112)
* feat: add getters for cachedb (#119)
* bump some lib versions.

# v1.3.1
date: 11.4.2022

Small fixes release.
* Empty keccak constant and remove access_list.clone (#111)
* chore: typo fixes
* fix is_static for Inspector initialize_interp

# v1.3.0
date: 30.4.2022

There are a lot of big changes that are included in this release as revm was integrated inside foundry.

* A lot of changed on Inspector, added new calls and flushed out how it should be called. Big effort mostly driven by Oliver Nordbjerg
* Big internal refactor and renaming: Machine->Inspector, call/create info are now in structs.
* feat: add serde support to model types. Thank you Matthias Seitz
* Added rust feature that sets memory limit on interpreter that is configurable with env.cfg. by Oliver Nordbjerg.
* Library bumped to higher version.

# v1.2.0
date 20.1.2022

Changes:
* Bump revm_precompile and added new feature for k256 lib.

# v1.1.0
date: 14.1.2022

There is bug introduced in last release with energy blcok optimization, it will crash revm if anywhere in contract is unknown OpCode. And now returning log after execution (ups) included them in eth/tests verification.

Changes:
* Bug fix for unknown OpCode
* Omit edgecase high nonce test. tracer energy fix 
* Some internal cleanup

# v1.0.0
date: 18.12.2021

It feel's like that the lib is in the state that is okay to promote it to the v1 version. Other that that, a lot of optimizations are done and the inspector trait was rewritten.

Changes: 
*  web3 db
*  precalculated energy blocks. Optimization
*  PC opcode as pointer. Optimization
*  U256 div_rem optimization
*  Inspector refactored and it is now closer to Host interface.

Optimization thread: https://github.com/bluealloy/revm/issues/7


# v0.5.0
date: 17.11.2021

A lot of optimization on machine(Interpreter) part, it is now at least 3x faster. On interface side, Error enum was renamed to Return and it is simplified. Additionally if needed energy measuring can be removed with rust feature.

Changes: 
* push instruction optimized.
* mload/mstore and memory optimized
* energy calculation optimized
* optimize i256
* switch stacks from H256 with U256
* Error's refactor to Return
* clippy/warnings/fmt cleanup
* Bump auto_impl to v0.5
* opcode renaming
* energy measurment can be removed with rust features.

# v0.4.1
date: 02.11.2021

Change in interface and how you can call evm. There is now multiple Database traits for use and inspector is taken on transact call as reference.

* 20ac70b - Database traits made useful.
* 46b5bcd - EVM Interface changed. Inspector called separately.


# v0.3.1
date: 27.10.2021

remove some warnings for unused imports and done cargo fmt.
# v0.3.0
date: 27.10.2021

Interface revamped and now looks a lot better.

Log:
* 1b1ebd8 - [revm] Interface. Inspector added, Env cleanup. revm-test passes (9 hours ago) <rakita>
* 351d4e0 - BIG interface change (11 hours ago) <rakita>
* a723827 - no_sdt to no_std (2 days ago) <rakita>
* a449bed - [precompiles] spelling, small cleanup (2 days ago) <rakita>


# v0.2.2

Same as v0.2.1 but added readme.
# v0.2.1
date: 25.10.2021

Big refactor, cleanup changes, and updating tests. EIP-3607 added.

Log:
* a6e01de - BIG reorg. workspace added. revm-precompile lib (20 minutes ago) <rakita>
* e50f6d3 - Move merkle trie from revm to eth/tests crate (4 hours ago) <rakita>
* 633ffd4 - Bump tests to v10.1 (28 hours ago) <rakita>
* 14b3de1 - Payment overflow check (30 hours ago) <rakita>
* 6e964ba - EIP-3607: Reject transactions from senders with deployed code (30 hours ago) <rakita>


# v0.2.0
date: 23.10.2021:

Published v0.2.0, first initial version of code. London supported and all eth state test are 100% passing or Istanbul/Berlin/London.


### 17.10.2021:
-For past few weeks working on this structure and project in general become really good and I like it. For me it surved as good distraction for past few weeks and i think i am going to get drained if i continue working on it, so i am taking break and i intend to come back after few months and finish it.
- For status:
    * machine/spec/opcodes/precompiles(without modexp) feels good and I probably dont need to touch them.
    * inspector: is what i wanted, full control on insides of EVM so that we can control it and modify it. will probably needs to add some small tweaks to interface but nothing major.
    * subroutines: Feels okay but it needs more scrutiny just to be sure that all corner cases are covered.
    * Test that are failing (~20) are mostly related to EIP-158: State clearing. For EIP-158 I will time to do it properly.
    * There is probably benefit of replaing HashMap hasher with something simpler, but this is research for another time.
## Project structure: