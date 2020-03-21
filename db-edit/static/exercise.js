function confirm_delete(id) {
    if (window.confirm("Sind Sie sicher? Es werden auch alle dazugehörigen Aufgeben mitgelöscht!")){
        fetch(`/manage/exercise/delete/${id}`, {
            method: 'delete'
        }).then(resq => location.reload()).catch(err => console.log(err));
    }
}
function toggle_edit(target, id) {
    const grandpa = target.parentElement.parentElement;
    const elem_form = grandpa.querySelector("form");
    const elem_a = grandpa.querySelector("a");
    const rename_btn = grandpa.querySelector(".btns_bar");
    if (elem_form.hidden){
        rename_btn.hidden = true
    }else{
        rename_btn.hidden = false
    }
    elem_form.hidden = !elem_form.hidden;
    elem_a.hidden = !elem_a.hidden;
}
