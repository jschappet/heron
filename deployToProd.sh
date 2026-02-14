#!/bin/zsh
set -e

# ----------------------------

# Config

# ----------------------------

APP_NAME="rv_api"
REMOTE="upload"
REMOTE_BASE="git/rv_api"
DATE="$(date +%Y-%m-%d)"
RELEASE_DIR="releases/$DATE"
CURRENT_LINK="$REMOTE_BASE/current"

# ----------------------------

# Build Linux binary via Docker

# ----------------------------

docker buildx build \
  --platform linux/amd64 -t rv_api .

docker rm rv_api-container 2>/dev/null || true

docker create --name rv_api-container --platform linux/amd64 rv_api
docker cp rv_api-container:/usr/src/app/target/release/heron ./rv_api
docker rm rv_api-container
#container run --rm -it \
#  --memory 8g \
#  --volume "$(pwd)":/usr/src/app \
#  --workdir /usr/src/app \
#  docker.io/library/rust:1.92 \
#  bash -c "
#    apt-get update && \
#    apt-get install -y libsqlite3-dev musl-tools pkg-config build-essential && \
#    source /usr/local/cargo/env && \
#    rustup target add x86_64-unknown-linux-gnu && \
#    rustup target add x86_64-unknown-linux-musl && \
#    cargo build --release --target x86_64-unknown-linux-musl
#  "



# ----------------------------

# Prepare remote release dir

# (mutable within the day)

# ----------------------------
echo "Making release Dir"
ssh $REMOTE "
mkdir -p $REMOTE_BASE/releases &&
rm -rf $REMOTE_BASE/$RELEASE_DIR &&
mkdir -p $REMOTE_BASE/$RELEASE_DIR
"

# ----------------------------

# Sync release artifacts

# ----------------------------

echo "Syncing Directories" 
rsync -a --progress  migrations/  $REMOTE:$REMOTE_BASE/$RELEASE_DIR/migrations
rsync -a --progress  templates/   $REMOTE:$REMOTE_BASE/$RELEASE_DIR/templates
echo "Sending rv_api"
rsync -a --progress  rv_api       $REMOTE:$REMOTE_BASE/$RELEASE_DIR/rv_api
echo "Sending restart.sh $REMOTE:$REMOTE_BASE/$RELEASE_DIR/ "
rsync -a --progress  restart.sh  $REMOTE:$REMOTE_BASE/$RELEASE_DIR/restart.sh

rsync -a --progress  $REMOTE:$REMOTE_BASE/local_database.db db_backup.db


# ----------------------------

# Activate release atomically

# ----------------------------

echo "Activating Release" 

ssh $REMOTE "cd $REMOTE_BASE; chmod +x $RELEASE_DIR/rv_api $RELEASE_DIR/restart.sh "

ssh $REMOTE "cd $REMOTE_BASE; ln -sfn $RELEASE_DIR/templates templates "
ssh $REMOTE "cd $REMOTE_BASE; ln -sfn $RELEASE_DIR/migrations migrations " 
ssh $REMOTE "cd $REMOTE_BASE; ln -sfn $RELEASE_DIR/rv_api rv_api "
ssh $REMOTE "cd $REMOTE_BASE; ln -sfn $RELEASE_DIR/restart.sh restart.sh "


# ----------------------------

# Restart service

# ----------------------------

ssh $REMOTE -x "
cd $REMOTE_BASE; nohup ./restart.sh >> rv_api.log 2>&1 &
"

echo "Deploy complete: $DATE"



ssh upload -x "
cd git/rv_api/; nohup ./restart.sh >> rv_api.log 2>&1 &
"