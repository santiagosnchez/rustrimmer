import random
import argparse


def random_sequence(length):
    """Generate a random DNA sequence of given length (fast, non-crypto)."""
    return "".join(random.choices("ACGT", k=length))


def quality_model(
    length,
    p_low: float = 0.25,
    p_high: float = 0.95,
    drop_prob: float = 0.02,
    edge_frac_min: float = 0.10,
    edge_frac_max: float = 0.15,
    n: int = 40,
):
    """Generate a realistic Phred+33 quality string using a binomial-like model.

    Parameters are exposed so callers can tune the model:
    - p_low: binomial p for the low-quality (edge) region.
    - p_high: binomial p for the high-quality region.
    - drop_prob: probability at each right-side position to draw from low-quality dist.
    - edge_frac_min/edge_frac_max: range for left-edge fraction of the read length.
    - n: number of Bernoulli trials approximating Phred max (default 40 -> 0..40).
    """
    # determine left-edge length (random between min and max fraction)
    edge_frac = random.uniform(edge_frac_min, edge_frac_max)
    edge_len = max(1, int(length * edge_frac))

    quals = []
    for i in range(length):
        if i < edge_len:
            # low-quality region
            q = sum(1 for _ in range(n) if random.random() < p_low)
        else:
            # mostly high quality, occasionally a low-quality draw
            if random.random() < drop_prob:
                q = sum(1 for _ in range(n) if random.random() < p_low)
            else:
                q = sum(1 for _ in range(n) if random.random() < p_high)
        # clamp to 0..40 just in case
        q = max(0, min(40, q))
        quals.append(chr(q + 33))
    return "".join(quals)


def sequence_id(index):
    """Generate a sequence ID based on the index."""
    return f"SEQ_{index:06d}"


def generate_fastq_records(
    read_length: int = 100,
    number: int = 1,
    p_low: float = 0.25,
    p_high: float = 0.95,
    drop_prob: float = 0.02,
    edge_frac_min: float = 0.10,
    edge_frac_max: float = 0.15,
    n: int = 40,
    bad_fraction: float = 0.0,
) -> None:
    """Print `number` random FASTQ records to stdout using the configurable quality model."""
    for i in range(1, number + 1):
        seq = random_sequence(read_length)
        # decide whether to make this record intentionally low-quality
        if bad_fraction > 0 and random.random() < bad_fraction:
            # produce a very poor-quality read (likely to be trimmed/dropped)
            qual = quality_model(
                read_length,
                p_low=0.05,
                p_high=0.15,
                drop_prob=0.8,
                edge_frac_min=0.4,
                edge_frac_max=0.6,
                n=n,
            )
        else:
            qual = quality_model(
                read_length,
                p_low=p_low,
                p_high=p_high,
                drop_prob=drop_prob,
                edge_frac_min=edge_frac_min,
                edge_frac_max=edge_frac_max,
                n=n,
            )
        print(f"@{sequence_id(i)}")
        print(seq)
        print("+")
        print(qual)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate random FASTQ records.")
    parser.add_argument(
        "--read_length", type=int, default=100, help="Length of each read."
    )
    parser.add_argument(
        "--number", type=int, default=1, help="Number of reads to generate."
    )
    parser.add_argument(
        "--p_low", type=float, default=0.25, help="Low-region binomial p."
    )
    parser.add_argument(
        "--p_high", type=float, default=0.95, help="High-region binomial p."
    )
    parser.add_argument(
        "--drop_prob",
        type=float,
        default=0.02,
        help="Probability of low-quality drop on right side.",
    )
    parser.add_argument(
        "--edge_min", type=float, default=0.10, help="Minimum left-edge fraction."
    )
    parser.add_argument(
        "--edge_max", type=float, default=0.15, help="Maximum left-edge fraction."
    )
    parser.add_argument(
        "--n", type=int, default=40, help="Binomial n (Phred max approximation)."
    )
    parser.add_argument(
        "--bad_fraction",
        type=float,
        default=0.0,
        help="Fraction of reads to make very low-quality (0.0-1.0).",
    )
    args = parser.parse_args()
    generate_fastq_records(
        read_length=args.read_length,
        number=args.number,
        p_low=args.p_low,
        p_high=args.p_high,
        drop_prob=args.drop_prob,
        edge_frac_min=args.edge_min,
        edge_frac_max=args.edge_max,
        n=args.n,
        bad_fraction=args.bad_fraction,
    )
