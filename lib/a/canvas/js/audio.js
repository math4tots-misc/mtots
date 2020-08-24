
// Direct Web Audio stuff
var actx = new (window.AudioContext || window.webkitAudioContext)();
function playseq(notes, nsecPerNote) {
    try {
        const startTime = actx.currentTime;
        const oscillator = actx.createOscillator();
        oscillator.type = 'triangle';
        for (var i = 0; i < notes.length; i++) {
            let note = noteToFreq(notes[i]);
            if (note) {
                oscillator.frequency.setValueAtTime(note, startTime + i * nsecPerNote);
            }
        }
        oscillator.connect(actx.destination);
        oscillator.start(startTime);
        oscillator.stop(startTime + notes.length * nsecPerNote);
    } catch (e) {
        external.invoke('debug/playseq-error/' + e);
    }
}

function noteToFreq(note) {
    if (typeof note === 'string') {
        const newNote = NOTE_TO_FREQ_MAP[note];
        return newNote === undefined ? note : newNote;
    } else {
        return note;
    }
}
