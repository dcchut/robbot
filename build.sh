docker build -t robbot:dummy .
docker create --name dummy robbot:dummy
docker cp dummy:target/release/robbot robbot
docker rm -f dummy
docker rm -f robbot:dummy