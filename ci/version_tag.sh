#!/bin/sh

VERSION=$(cargo metadata --no-deps --format-version 1 | jq '.packages[0].version' | sed -e 's/"\(.*\)"/\1/')

if ( git tag ${VERSION} > /dev/null 2>&1 ) ; then
    echo "Pushing tag ${VERSION}"
    git push origin ${VERSION}
else
    echo "Tag ${VERSION} already exists"
fi

