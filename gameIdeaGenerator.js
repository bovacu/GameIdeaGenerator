globalInstance = undefined;
const importObject = {
  env: {
    random_range: generateRandom
  }
};

(async () => {
  let response = await fetch('target/wasm32-unknown-unknown/release/game_idea_generator.wasm');
  let bytes = await response.arrayBuffer();
  let { instance } = await WebAssembly.instantiate(bytes, importObject);
  globalInstance = instance;

  getRandomIdea();
})();

function getRandomIdea() {
  const linearMemory = globalInstance.exports.memory;

  const offset = globalInstance.exports.generate_random_idea();
  const stringBuffer = new Uint8Array(linearMemory.buffer, offset,
    globalInstance.exports.get_idea_size());

  // create a string from this buffer
  let str = '';
  for (let i=0; i<stringBuffer.length; i++) {
    str += String.fromCharCode(stringBuffer[i]);
  }

  document.getElementById('idea').innerText = str;
}

function generateRandom(min = 0, max = 100) {
  let difference = max - min;
  let rand = Math.random();
  rand = Math.floor( rand * difference);
  rand = rand + min;
  return rand;
}

function copyToClipboard() {
  navigator.clipboard.writeText(document.getElementById('idea').innerText);
  var x = document.getElementById("copied-clipboard");
  x.className = "show";
  setTimeout(function(){ x.className = x.className.replace("show", ""); }, 3000);
}