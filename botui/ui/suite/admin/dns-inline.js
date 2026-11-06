    function updateTargetPlaceholder() {
        const recordType = document.getElementById("record-type").value;
        const targetInput = document.getElementById("target-input");
        const targetHelp = document.getElementById("target-help");

        switch (recordType) {
            case "A":
                targetInput.placeholder = "192.168.1.1";
                targetHelp.textContent = window.t
                    ? window.t("admin-dns-target-help-a")
                    : "Enter the IPv4 address to point to";
                break;
            case "AAAA":
                targetInput.placeholder = "2001:db8::1";
                targetHelp.textContent = window.t
                    ? window.t("admin-dns-target-help-aaaa")
                    : "Enter the IPv6 address to point to";
                break;
            case "CNAME":
                targetInput.placeholder = "target.example.com";
                targetHelp.textContent = window.t
                    ? window.t("admin-dns-target-help-cname")
                    : "Enter the target domain name";
                break;
        }
    }

    function openRemoveModal(hostname, id) {
        document.getElementById("remove-hostname-name").textContent = hostname;
        document
            .getElementById("confirm-remove-btn")
            .setAttribute("hx-vals", JSON.stringify({ id: id }));
        htmx.process(document.getElementById("confirm-remove-btn"));
        document.getElementById("remove-dns-modal").showModal();
    }

    function openEditModal(id) {
        htmx.ajax("GET", "/api/dns/" + id + "/edit", {
            target: "#edit-dns-form-container",
            swap: "innerHTML",
        });
        document.getElementById("edit-dns-modal").showModal();
    }
