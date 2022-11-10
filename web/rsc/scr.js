function getSelectedSpeaker() {
    var selection = document.getElementById('device').value;
    return selection
}

function fileButtonHandler(obj) {
    var loc = document.getElementById('info-bar').innerHTML + obj.slice(5,);
    if (loc.endsWith('/')) {
        getFiles(loc)
    } else {
        getSpeaker("", loc)
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
    var str = "<ul>";
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

function play() {
    var speaker = getSelectedSpeaker();
}

function chgVlm(inc) {
    var speaker = getSelectedSpeaker()
    const xhr = new XMLHttpRequest();
    var cmd = inc ? "v-inc" : "v-dec"
    xhr.open("GET", `http://${location.host}/api/control/playback/${speaker}/${cmd}`)
    xhr.send()
    // When the Request is completed
    xhr.obload = function() {
        if (xhr.status === 200) {
            data = JSON.parse(xhr.responseText)
            console.log(data.count)
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
        if (e.lengthComputable) {
            console.log(`${e.loaded} B of ${e.total} B loaded...`)
        } else {
            console.log(`${e.loaded} B loaded...`)
        }
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
