#!/bin/bash

##
#  script to install portman in the specified directory.
#  e.g.
#    ./install /usr/opt/portman
#
#   creates:
#      /usr/opt/portman  - if necessary
#      /usr/opt/portman/bin  - if necessary.
#   If there's no target/release/portman
#     runs cargo build --target=release to create it.
#   Installs target/release/portman -> /usr/opt/portman/bin/portman.
#

PREFIX=$1
if test "$PREFIX" = ""
then
    echo "Usage:"
    echo "   ./install installation-directory-prefix"
    exit -1
fi

echo Checking for portman binary:

if  test !  -x target/release/portman
then
    echo Release target of portman not yet built -- attemptint to build:
    cargo build --release
    if test "$?" != "0"
    then
	echo "Build of portman failed..fix problems and try again."
	exit -1
    else
	echo "portman release target successfully built"
    fi
 fi

echo Creating directory tree:

install -d $PREFIX/bin

echo Installing:

install -m 0755  target/release/portman $PREFIX/bin
if test  "$?" = "0"
then
    echo portman installed in $PREFIX/bin/portman
    exit 0
else
    echo Could not install portman -- check error messages, fix problems and 
    echo try again.
    exit -1
fi
