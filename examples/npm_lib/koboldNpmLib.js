// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { u8aToU8a } = require('@polkadot/util');

export async function koboldNpmLib() {
    const wsProvider = new WsProvider('wss://rpc.polkadot.io');
    const api = await ApiPromise.create({ provider: wsProvider });
    const hexPrefixed = api.genesisHash.toHex();
    return u8aToU8a(hexPrefixed); // Uint8Array
}
