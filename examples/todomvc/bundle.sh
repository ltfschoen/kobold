# Rename the built dist file
mv ./dist/kobold_todomvc_example.js ./dist/kobold_todomvc_example_large.js
# Minimize the dist file into a new one
node_modules/.bin/esbuild --bundle ./dist/kobold_todomvc_example_large.js --outfile=./dist/kobold_todomvc_example.js --format=esm
