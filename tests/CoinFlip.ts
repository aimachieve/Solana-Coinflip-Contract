import * as anchor from "@project-serum/anchor";

describe("CoinFlip", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  it("Is initialized!", async () => {
    // Add your test here.
    const TREASURY_TAG = Buffer.from("coin-flip-vault");
    const wrappedSolAccountAddr = new anchor.web3.PublicKey("So11111111111111111111111111111111111111112");
    const treasuryKey = await pda([TREASURY_TAG, wrappedSolAccountAddr.toBuffer()], new anchor.web3.PublicKey("FMgTFH3VJUfZVqGoqjjcskrxKWU3MUkCGV5NtnF6MYa1"));
    console.log(treasuryKey.toString());

    //7iSTfbxiAbyntogaw1vQv6X5gtZpyuLr9gAwahJ6uxjj
  });
});

export const pda = async (
  seeds: (Buffer | Uint8Array)[],
  programId: anchor.web3.PublicKey
) => {
  const [pdaKey] = await anchor.web3.PublicKey.findProgramAddress(
    seeds,
    programId
  );
  return pdaKey;
}
