#combine
echo "combine process start...."
cat rag_agentic.tar.gz.part* > rag_agentic_combine.tar.gz

#verify
echo "verify process...."
md5sum -c rag_agentic.tar.gz.md5
