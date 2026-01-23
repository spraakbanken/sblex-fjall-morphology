import json
from tempfile import TemporaryDirectory

import pytest
from sblex_fjall_morphology import FjallMorphology
from syrupy.assertion import SnapshotAssertion
from syrupy.extensions.json import JSONSnapshotExtension


@pytest.fixture
def snapshot_json(snapshot: SnapshotAssertion) -> SnapshotAssertion:
    return snapshot.with_defaults(extension_class=JSONSnapshotExtension)


def test_build_and_load_morphology(snapshot_json: SnapshotAssertion) -> None:
    with TemporaryDirectory(prefix="test.db") as dir:
        morph = FjallMorphology(dir)
        morph.build_from_path("assets/testing/saldo.lex")

        result = morph.lookup("dv")
        assert result is None

        result = morph.lookup("dväljs")
        result_json = json.loads(result)
        assert result_json == snapshot_json

        result = morph.lookup_with_cont("dv")
        result_json = json.loads(result)
        assert result_json == snapshot_json

        result = morph.lookup_with_cont("dväljs")
        result_json = json.loads(result)
        assert result_json == snapshot_json
