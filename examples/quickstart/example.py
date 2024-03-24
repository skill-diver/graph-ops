import ast
from argparse import ArgumentParser, Namespace
from os import path as osp
from timeit import default_timer

from tqdm import tqdm

import ofnil
from ofnil.infra.graph.hop_collectors.pyg_hop_collector import PyGHopCollector

home = osp.dirname(osp.abspath(__file__))


def get_args(argv=None) -> Namespace:
    parser = ArgumentParser("Quickstart Neighbor Sampling Benchmarking")

    arg = parser.add_argument
    arg(
        "-b",
        "--batch_sizes",
        nargs="+",
        default=[4096, 2048, 1024, 512],
        type=int,
        help="Training batch size.",
    )
    arg(
        "-f",
        "--fan_outs",
        default=[[10, 5], [15, 10, 5], [20, 15, 10]],
        type=ast.literal_eval,
        help="Neighbor sampling fanout.",
    )
    arg("-r", "--runs", default=1, help="Number of runs")

    return parser.parse_args(argv)


def run(args: Namespace):
    client = ofnil.Client(home)
    for fanout in args.fan_outs:
        for batch_size in args.batch_sizes:
            print(f"Sampling with {fanout} neighbors, batch size = {batch_size}")
            # TODO(tatiana): compute aggregations for Reviewers instead
            # TODO(kaili): PyG interface now. Will support DGL and PyG later.
            disjoint = False
            sparse = True
            loader = client.neighbor_sampled_dataloader(
                "default/GraphDataset/fraud_detection_train_dataset",
                hop_collector=PyGHopCollector(disjoint=disjoint, sparse=sparse),
                input_nodes="Product",
                sample_with_replacement=True,
                num_neighbors=fanout,
                batch_size=batch_size,
            )
            runtimes = []
            num_iterations = 0
            for _ in range(args.runs):
                start = default_timer()
                for _ in tqdm(loader):
                    num_iterations += 1
                stop = default_timer()
                runtimes.append(round(stop - start, 3))
            average_time = round(sum(runtimes) / args.runs, 3)
            print(
                f"batch size={batch_size}, iterations={num_iterations}, "
                f"runtimes={runtimes}, average runtime={average_time}"
            )


if __name__ == "__main__":
    args = get_args()
    run(args)
