function getSelectedSpeaker() {
    var selection = document.getElementById('device').value;
    return selection
}

function fileButtonHandler(obj) {
    var loc = document.getElementById('info-bar').innerHTML + obj.slice(5,);
    console.log(loc)
    if (loc.endsWith('/')) {
        getFiles(loc)
    } else {
        if (loc.startsWith('/')) {
            loc = loc.slice(1,);
        }
        playFile(loc)
        //getSpeaker("", loc)
    }
}
function fileButtonBack() {
    var pathnow = document.getElementById('info-bar').innerHTML;
    if (pathnow.endsWith('/')) {
        var overhead = -2;
    } else {
        var overhead = -1;
    }
    var path = pathnow.split('/').slice(0,overhead).join('/') + '/';
    getFiles(path)
}
function speakerButtonHandler(obj, path, action) {
    var spk = obj.slice(8,);
    const xhr = new XMLHttpRequest();
    if (path.startsWith('/')) {
        path = path.slice(1,);
    }
    var url = `http://${location.host}/api/control/play/${spk}/${path}`
    if (action === "n") {
        var url = `http://${location.host}/api/control/next/${spk}/${path}`
    }
    xhr.open("GET", url)
    xhr.send()
    // When the Request is completed
    xhr.onload = function() {
        if (xhr.status === 200) {
            getFiles("")
        } else {
            console.log("Failed to Play")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to get response")
    }
    // Log progress
    xhr.onprogress = function(e) {
        if (e.lengthComputable) {
            console.log(`${e.loaded} B of ${e.total} B loaded...`)
        } else {
            console.log(`${e.loaded} B loaded...`)
        }
    }
}

function writeFiles(data, loc) {
    document.getElementById("description").innerHTML = "Available music files";
    document.getElementById("info-bar").innerHTML = loc;
    var str = "<ul class=\"p-0\">";
    var last_char = "";
    data.sort();
    for (var i = 0; i < data.length; i++) {
        var jumper = "";
        var e = data[i]
        var first_char = e.toLowerCase()[0]
        if (last_char !== first_char) {
            last_char = first_char
            jumper = `<span class='marker' id='${first_char}'></span>`
        }
        str += "<li><button onclick='fileButtonHandler(this.id)' " + `id='file-${e}'>` + e + "</button>" + jumper + "</li>"
    }
    str += "</ul>"
    document.getElementById("datalist").innerHTML = str;

    let markers = ["<a href='#○'>○</a>"];
    let ms = document.getElementsByClassName("marker");
    for (var i = 0; i < ms.length; i++) {
        markers.push(`<a href='#${ms[i].id}'>${ms[i].id.toUpperCase()}</a>`);
    }
    document.getElementById("marker-list").innerHTML = markers.join("<br>");
}

function getFiles(loc) {
    const xhr = new XMLHttpRequest()
    if (!loc.startsWith("/")) {
        var path = "/" + loc;
    } else {
        var path = loc;
    }
    xhr.open("GET", `http://${location.host}/api/filelist${path}`)
    xhr.send()
    // When the Request is completed
    xhr.onload = function() {
        if (xhr.status === 200) {
            data = JSON.parse(xhr.responseText)
            console.log(data.count)
            writeFiles(data, path)
        } else {
            console.log("No filelist available")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to get filelist")
    }
    // Log progress
    xhr.onprogress = function(e) {
        if (e.lengthComputable) {
            console.log(`${e.loaded} B of ${e.total} B loaded...`)
        } else {
            console.log(`${e.loaded} B loaded...`)
        }
    }
}

function writeSpeaker(data, path) {
    document.getElementById("description").innerHTML = "Available speakers";
    document.getElementById("info-bar").innerHTML = path;
    var str = "<ul>";
    for (var i = 0; i < data.length; i++) {
        var e = data[i];
        str += `<li>${e.ip} (${e.trackname} [${e.trackelapsed} / ${e.trackduration})s] ` +
            `<button class='action' onclick='speakerButtonHandler("speaker-${e.ip}", "${path}", "p")'>Play</button> ` +
            `<button class='action' onclick='speakerButtonHandler("speaker-${e.ip}", "${path}", "n")'>Next</button> ` +
            "</li>"
    }
    str += "</ul>"
    document.getElementById("datalist").innerHTML = str;
    document.getElementById("marker-list").innerHTML = "<a href='#○'>○</a>";
}

function updateSpeakerList() {
    const xhr = new XMLHttpRequest();
    xhr.open("GET", `http://${location.host}/api/speakers/`)
    xhr.send()
    // When the Request is completed
    xhr.onload = function() {
        if (xhr.status === 200) {
            data = JSON.parse(xhr.responseText)
            console.log(data.count)

            obj = document.getElementById("device");
            for (e of data) {
                obj.innerHTML += `<option value="${e.ip}">${e.ip}</option>\n`
            }
        } else {
            console.log("No speaker available")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to get speaker")
    }
    // Log progress
    xhr.onprogress = function(e) {
        if (e.lengthComputable) {
            console.log(`${e.loaded} B of ${e.total} B loaded...`)
        } else {
            console.log(`${e.loaded} B loaded...`)
        }
    }
}

function getSpeaker(spk, path) {
    const xhr = new XMLHttpRequest();
    xhr.open("GET", `http://${location.host}/api/speakers/${spk}`)
    xhr.send()
    // When the Request is completed
    xhr.onload = function() {
        if (xhr.status === 200) {
            data = JSON.parse(xhr.responseText)
            console.log(data.count)
            writeSpeaker(data, path)
        } else {
            console.log("No speaker available")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to get speaker")
    }
    // Log progress
    xhr.onprogress = function(e) {
        if (e.lengthComputable) {
            console.log(`${e.loaded} B of ${e.total} B loaded...`)
        } else {
            console.log(`${e.loaded} B loaded...`)
        }
    }
}

function playFile(loc) {
    var speaker = getSelectedSpeaker();

    const xhr = new XMLHttpRequest();
    xhr.open("GET", `http://${location.host}/api/control/play/${speaker}/${loc}`)
    xhr.send()
    // When the Request is completed
    xhr.onload = function() {
        if (xhr.status === 200) {
            console.log(".")
            updateSpeaker()
        } else {
            console.log("Can not play file")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to play file")
    }
    // Log progress
    xhr.onprogress = function(e) {
        if (e.lengthComputable) {
            console.log(`${e.loaded} B of ${e.total} B loaded...`)
        } else {
            console.log(`${e.loaded} B loaded...`)
        }
    }
}

function updateSpeaker() {
    var speaker = getSelectedSpeaker();

    const xhr = new XMLHttpRequest();
    xhr.open("GET", `http://${location.host}/api/speakers/${speaker}`);
    xhr.send()
    xhr.onload = function() {
        if (xhr.status === 200) {
            data = JSON.parse(xhr.responseText)
            var spk = data[0];
            document.getElementById("title").innerHTML = spk.trackname;
            document.getElementById("prog-bar").style.width = `${(spk.trackelapsed / spk.trackduration) * 100}%`;
            document.getElementById("play").innerHTML = spk.is_playing ? "Pause" : "Play";
        } else {
            console.log("No speaker available")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to get speaker")
    }
    // Log progress
    xhr.onprogress = function(e) {
        return
    }
}

function play() {
    var btn = document.getElementById("play");
    var cmd = btn.innerHTML.toLowerCase();

    var speaker = getSelectedSpeaker();
    const xhr = new XMLHttpRequest()
    xhr.open("GET", `http://${location.host}/api/control/playback/${speaker}/${cmd}`)
    xhr.send()
    // When the Request is completed
    xhr.onload = function() {
        if (xhr.status === 200) {
            console.log(".")
            updateSpeaker()
        } else {
            console.log("Error while trying to set playback")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to set playback")
    }
    // Log progress
    xhr.onprogress = function(e) {
        if (e.lengthComputable) {
            console.log(`${e.loaded} B of ${e.total} B loaded...`)
        } else {
            console.log(`${e.loaded} B loaded...`)
        }
    }
}

function chgVlm(inc) {
    var speaker = getSelectedSpeaker()
    const xhr = new XMLHttpRequest();
    var cmd = inc ? "v-inc" : "v-dec"
    xhr.open("GET", `http://${location.host}/api/control/playback/${speaker}/${cmd}`)
    xhr.send()
    // When the Request is completed
    xhr.onload = function() {
        if (xhr.status === 200) {
            return
        } else {
            console.log("Error while changing volume")
        }
    }
    // When a network error is encountered
    xhr.onerror = function() {
        console.log("Failed to set volume")
    }
    // Log progress
    xhr.onprogress = function(e) {
        return
    }
}

function vlmUp() {
    chgVlm(true)
}
function vlmDw() {
    chgVlm(false)
}

// Make a request to the Backend and ask for a filelist
getFiles("")
updateSpeakerList()
updateSpeaker()
// Update the speakers state periodically
var interval_1 = setInterval(function() {
    updateSpeaker();
}, 1000);
// Use keyboard for some functions
window.addEventListener("keydown", function (event) {
  if (event.defaultPrevented) {
    return; // Do nothing if the event was already processed
  }

  switch (event.key) {
    case "ArrowDown":
      vlmDw();
      break;
    case "ArrowUp":
      vlmUp();
      break;
    case "P":
    case "p":
      play();
      break;
    default:
      return; // Quit when this doesn't handle the key event.
  }

  // Cancel the default action to avoid it being handled twice
  event.preventDefault();
}, true);
// the last option dispatches the event to the listener first,
// then dispatches event to window
