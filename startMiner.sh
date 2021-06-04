cd ol/cli
 cargo run -- --swarm-path=/home/teja9999/libra/swarm_temp --swarm-persona=alice init
mkdir ../../swarm_temp/0/blocks
cp ../fixtures/blocks/test/alice/* ../../swarm_temp/0/blocks/
NODE_ENV="test" cargo run -p miner -- --swarm-path=/home/teja9999/libra/swarm_temp --swarm-persona=alice start
