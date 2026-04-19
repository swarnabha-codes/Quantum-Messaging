/**
 * BB84 Quantum Key Distribution Protocol Implementation
 *
 * This is a SIMULATION of the BB84 protocol using JavaScript.
 * In a real quantum system, qubits would be transmitted; here we simulate
 * the quantum states with classical bits and bases.
 *
 * Protocol Steps:
 * 1. Alice prepares random bits in random bases (rectilinear or diagonal)
 * 2. Alice sends qubits to Bob (simulated)
 * 3. Bob measures with random bases (may not match Alice's)
 * 4. Alice and Bob publicly compare bases (NOT the bits)
 * 5. They keep only measurements where bases matched (sifting)
 * 6. They verify a random subset for eavesdropping detection
 * 7. Remaining bits form the shared secret key
 */

export class BB84 {
  private keyLength: number;
  private bases: string[];

  constructor(keyLength = 256) {
    this.keyLength = keyLength;
    this.bases = ['rectilinear', 'diagonal']; // + and x polarizations
  }

  /**
   * Generate random bit (0 or 1)
   */
  private randomBit(): number {
    return Math.random() < 0.5 ? 0 : 1;
  }

  /**
   * Generate random base selection
   */
  private randomBase(): string {
    return this.bases[Math.floor(Math.random() * 2)];
  }

  /**
   * Alice's preparation phase
   * Creates random bits with random basis choices
   */
  alicePrepare(): { bits: number[], bases: string[] } {
    const aliceBits: number[] = [];
    const aliceBases: string[] = [];

    for (let i = 0; i < this.keyLength; i++) {
      aliceBits.push(this.randomBit());
      aliceBases.push(this.randomBase());
    }

    return { bits: aliceBits, bases: aliceBases };
  }

  /**
   * Bob's measurement phase
   * Measures qubits with randomly chosen bases
   * If basis doesn't match, measurement is random
   */
  bobMeasure(aliceBits: number[], aliceBases: string[]): { bits: number[], bases: string[] } {
    const bobBases: string[] = [];
    const bobBits: number[] = [];

    for (let i = 0; i < this.keyLength; i++) {
      const bobBase = this.randomBase();
      bobBases.push(bobBase);

      if (bobBase === aliceBases[i]) {
        // Correct basis: Bob measures Alice's bit correctly
        bobBits.push(aliceBits[i]);
      } else {
        // Wrong basis: Bob gets random measurement
        bobBits.push(this.randomBit());
      }
    }

    return { bits: bobBits, bases: bobBases };
  }

  /**
   * Basis Sifting Phase
   * Alice and Bob compare bases publicly and keep only matching ones
   * The actual bit values remain secret
   */
  siftBases(aliceBases: string[], bobBases: string[]): number[] {
    const siftedIndices: number[] = [];

    for (let i = 0; i < this.keyLength; i++) {
      if (aliceBases[i] === bobBases[i]) {
        siftedIndices.push(i);
      }
    }

    return siftedIndices;
  }

  /**
   * Generate shared key from sifted bits
   */
  generateKey(aliceBits: number[], bobBits: number[], siftedIndices: number[]): string {
    let key = '';
    for (const index of siftedIndices) {
      if (aliceBits[index] === bobBits[index]) {
        key += aliceBits[index].toString();
      }
    }
    // Take first 32 characters for 256-bit key (but since memo is 32 bytes, we can use more)
    return key.slice(0, 256);
  }

  /**
   * Simulate full BB84 protocol
   */
  simulateKeyExchange(): string {
    // Alice prepares
    const alice = this.alicePrepare();

    // Bob measures
    const bob = this.bobMeasure(alice.bits, alice.bases);

    // Sift bases
    const siftedIndices = this.siftBases(alice.bases, bob.bases);

    // Generate key
    const key = this.generateKey(alice.bits, bob.bits, siftedIndices);

    return key;
  }
}