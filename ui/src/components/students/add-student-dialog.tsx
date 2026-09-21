import { useState, useRef, useEffect } from "react";
import {
  rpc,
  type StudentDetail,
  type GuardianSearchHit,
  type CreateStudentInput,
  type GuardianLinkInput,
} from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  UserPlus,
  AlertTriangle,
  Loader2,
  Search,
  CheckCircle2,
  X,
  Camera,
} from "lucide-react";

export interface AddStudentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (student: StudentDetail) => void;
}

export function AddStudentDialog({
  open,
  onOpenChange,
  onSuccess,
}: AddStudentDialogProps) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Phone conflict warning state
  const [conflictWarning, setConflictWarning] = useState<string | null>(null);
  const [pendingPayload, setPendingPayload] = useState<CreateStudentInput | null>(null);

  // Student details
  const [name, setName] = useState("");
  const [nameBn, setNameBn] = useState("");
  const [gender, setGender] = useState("male");
  const [dob, setDob] = useState("");
  const [bloodGroup, setBloodGroup] = useState("");
  const [religion, setReligion] = useState("Islam");
  const [nidBirthCert, setNidBirthCert] = useState("");
  const [phone, setPhone] = useState("");
  const [emergencyPhone, setEmergencyPhone] = useState("");
  const [email, setEmail] = useState("");
  const [presentAddress, setPresentAddress] = useState("");
  const [permanentAddress, setPermanentAddress] = useState("");
  const [district, setDistrict] = useState("");
  const [thana, setThana] = useState("");
  const [classLevel, setClassLevel] = useState("");
  const [roll, setRoll] = useState("");
  const [groupMajor, setGroupMajor] = useState("General");
  const [previousSchool, setPreviousSchool] = useState("");
  const [admissionDate, setAdmissionDate] = useState(() =>
    new Date().toISOString().slice(0, 10)
  );
  const [customCode, setCustomCode] = useState("");
  const [useCustomCode, setUseCustomCode] = useState(false);
  const [notes, setNotes] = useState("");

  // Photo
  const [photoBase64, setPhotoBase64] = useState<string | null>(null);
  const [photoVaultId, setPhotoVaultId] = useState<string | null>(null);
  const [uploadingPhoto, setUploadingPhoto] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Guardian mode: 'new' | 'existing'
  const [guardianMode, setGuardianMode] = useState<"new" | "existing">("new");
  const [guardianSearchQuery, setGuardianSearchQuery] = useState("");
  const [guardianSearchResults, setGuardianSearchResults] = useState<GuardianSearchHit[]>(
    []
  );
  const [searchingGuardians, setSearchingGuardians] = useState(false);
  const [selectedGuardian, setSelectedGuardian] = useState<GuardianSearchHit | null>(
    null
  );

  // New Guardian fields
  const [gName, setGName] = useState("");
  const [gNameBn, setGNameBn] = useState("");
  const [gRelationship, setGRelationship] = useState("father");
  const [gPhone, setGPhone] = useState("");
  const [gPhone2, setGPhone2] = useState("");
  const [gEmail, setGEmail] = useState("");
  const [gNid, setGNid] = useState("");
  const [gOccupation, setGOccupation] = useState("");
  const [gAddress, setGAddress] = useState("");

  const [isPrimary, setIsPrimary] = useState(true);
  const [receivesSms, setReceivesSms] = useState(true);

  // Reset form when opened
  useEffect(() => {
    if (open) {
      setName("");
      setNameBn("");
      setGender("male");
      setDob("");
      setBloodGroup("");
      setReligion("Islam");
      setNidBirthCert("");
      setPhone("");
      setEmergencyPhone("");
      setEmail("");
      setPresentAddress("");
      setPermanentAddress("");
      setDistrict("");
      setThana("");
      setClassLevel("");
      setRoll("");
      setGroupMajor("General");
      setPreviousSchool("");
      setAdmissionDate(new Date().toISOString().slice(0, 10));
      setCustomCode("");
      setUseCustomCode(false);
      setNotes("");

      setPhotoBase64(null);
      setPhotoVaultId(null);

      setGuardianMode("new");
      setGuardianSearchQuery("");
      setGuardianSearchResults([]);
      setSelectedGuardian(null);

      setGName("");
      setGNameBn("");
      setGRelationship("father");
      setGPhone("");
      setGPhone2("");
      setGEmail("");
      setGNid("");
      setGOccupation("");
      setGAddress("");
      setIsPrimary(true);
      setReceivesSms(true);

      setError(null);
      setConflictWarning(null);
      setPendingPayload(null);
    }
  }, [open]);

  // Debounced guardian search
  useEffect(() => {
    if (guardianMode !== "existing" || guardianSearchQuery.trim().length < 2) {
      setGuardianSearchResults([]);
      return;
    }

    const timer = setTimeout(async () => {
      setSearchingGuardians(true);
      try {
        const res = await rpc("guardians.search", {
          query: guardianSearchQuery.trim(),
          limit: 8,
        });
        if (res.ok && res.data) {
          setGuardianSearchResults(res.data.matches || []);
        }
      } catch (e) {
        console.error("Failed to search guardians", e);
      } finally {
        setSearchingGuardians(false);
      }
    }, 200);

    return () => clearTimeout(timer);
  }, [guardianSearchQuery, guardianMode]);

  // Handle Photo selection
  const handlePhotoSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    if (file.size > 20 * 1024 * 1024) {
      setError("Photo size cannot exceed 20MB");
      return;
    }

    const reader = new FileReader();
    reader.onload = async () => {
      const b64 = reader.result as string;
      setPhotoBase64(b64);
      setUploadingPhoto(true);

      try {
        const res = await rpc("vault.upload_file", {
          owner_type: "student",
          owner_id: "temp_student_avatar",
          kind: "photo",
          file_name: file.name,
          mime_type: file.type || "image/jpeg",
          data_base64: b64,
        });

        if (res.ok && res.data) {
          setPhotoVaultId(res.data.id);
        } else {
          setError(res.error?.message || "Failed to process photo upload");
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setUploadingPhoto(false);
      }
    };
    reader.readAsDataURL(file);
  };

  const buildStudentPayload = (): CreateStudentInput => {
    const guardians: GuardianLinkInput[] = [];

    if (guardianMode === "existing" && selectedGuardian) {
      guardians.push({
        guardian_id: selectedGuardian.id,
        relationship: selectedGuardian.relationship,
        is_primary: isPrimary,
        receives_sms: receivesSms,
        new_guardian: null,
      });
    } else {
      guardians.push({
        guardian_id: null,
        relationship: gRelationship.toLowerCase(),
        is_primary: isPrimary,
        receives_sms: receivesSms,
        new_guardian: {
          name: gName.trim(),
          name_bn: gNameBn.trim() || null,
          relationship: gRelationship.toLowerCase(),
          occupation: gOccupation.trim() || null,
          monthly_income_poisha: null,
          nid: gNid.trim() || null,
          phone: gPhone.trim(),
          phone2: gPhone2.trim() || null,
          whatsapp_number: null,
          email: gEmail.trim() || null,
          present_address: gAddress.trim() || presentAddress.trim() || null,
          permanent_address: null,
          notes: null,
        },
      });
    }

    return {
      student_code: useCustomCode && customCode.trim() ? customCode.trim() : null,
      name: name.trim(),
      name_bn: nameBn.trim() || null,
      gender,
      dob,
      blood_group: bloodGroup || null,
      religion: religion || null,
      nid_or_birth_cert: nidBirthCert.trim() || null,
      phone: phone.trim(),
      emergency_phone: emergencyPhone.trim() || null,
      email: email.trim() || null,
      present_address: presentAddress.trim(),
      permanent_address: permanentAddress.trim() || null,
      district: district.trim() || null,
      thana: thana.trim() || null,
      previous_school: previousSchool.trim() || null,
      current_school_roll: roll.trim() || null,
      class_level: classLevel.trim() || null,
      group_major: groupMajor || null,
      photo_vault_id: photoVaultId,
      admission_date: admissionDate || null,
      notes: notes.trim() || null,
      guardians,
    };
  };

  const handleSubmit = async (e: React.FormEvent, allowDuplicate = false) => {
    e.preventDefault();

    if (!name.trim()) {
      setError("Student name is required");
      return;
    }
    if (!dob) {
      setError("Date of birth is required");
      return;
    }
    if (!phone.trim()) {
      setError("Student phone number is required");
      return;
    }
    if (!presentAddress.trim()) {
      setError("Present address is required");
      return;
    }

    if (guardianMode === "existing" && !selectedGuardian) {
      setError("Please search and select an existing guardian or switch to Add New");
      return;
    }
    if (guardianMode === "new") {
      if (!gName.trim()) {
        setError("Guardian name is required");
        return;
      }
      if (!gPhone.trim()) {
        setError("Guardian phone is required");
        return;
      }
    }

    const payload = pendingPayload || buildStudentPayload();
    setSubmitting(true);
    setError(null);

    try {
      const res = await rpc("students.create", {
        student: payload,
        allow_duplicate_phone: allowDuplicate,
      });

      if (res.ok && res.data) {
        onSuccess(res.data);
        onOpenChange(false);
      } else if (res.error?.code === "phone_conflict_warning") {
        setConflictWarning(res.error.message);
        setPendingPayload(payload);
      } else {
        setError(res.error?.message || "Failed to admit student");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[92vh] overflow-y-auto p-6">
        <DialogHeader>
          <DialogTitle className="text-xl font-bold flex items-center gap-2">
            <UserPlus className="h-5 w-5 text-primary" />
            Admit New Student
          </DialogTitle>
        </DialogHeader>

        {/* Phone Conflict Warning Modal Overlay */}
        {conflictWarning && (
          <div className="p-4 rounded-[var(--radius-md)] bg-warning-subtle border-2 border-warning text-foreground space-y-3 animate-in fade-in">
            <div className="flex items-start gap-3">
              <AlertTriangle className="h-6 w-6 text-warning shrink-0" />
              <div>
                <h3 className="text-sm font-bold text-warning-text">
                  Duplicate Phone Warning
                </h3>
                <p className="text-xs mt-1 text-foreground">{conflictWarning}</p>
                <p className="text-[11px] text-muted-foreground mt-1">
                  If this student is a sibling or family member sharing the same household
                  number, you can proceed with registration.
                </p>
              </div>
            </div>
            <div className="flex justify-end gap-2 pt-2 border-t border-warning-border">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => {
                  setConflictWarning(null);
                  setPendingPayload(null);
                }}
              >
                Edit Phone Number
              </Button>
              <Button
                type="button"
                size="sm"
                className="bg-warning-text hover:bg-warning-text/90 text-white"
                onClick={(e) => void handleSubmit(e, true)}
                disabled={submitting}
              >
                {submitting ? "Admitting..." : "Proceed Anyway (Allow Shared Phone)"}
              </Button>
            </div>
          </div>
        )}

        <form onSubmit={(e) => void handleSubmit(e, false)} className="space-y-6 pt-2">
          {error && (
            <div className="p-3 text-sm rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          {/* Section 1: Student Information & Photo */}
          <div className="space-y-4">
            <div className="flex items-center justify-between border-b border-border pb-2">
              <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground">
                1. Student Information
              </h3>
              <div className="flex items-center gap-2">
                <label className="text-xs text-muted-foreground flex items-center gap-1.5 cursor-pointer select-none">
                  <input
                    type="checkbox"
                    checked={useCustomCode}
                    onChange={(e) => setUseCustomCode(e.target.checked)}
                    className="rounded text-primary focus:ring-primary"
                  />
                  Custom Student Code
                </label>
              </div>
            </div>

            <div className="flex flex-col md:flex-row gap-6 items-start">
              {/* Photo Box */}
              <div className="flex flex-col items-center gap-2 shrink-0 w-full md:w-32">
                <div
                  onClick={() => fileInputRef.current?.click()}
                  className="relative w-32 h-36 rounded-[var(--radius-md)] border-2 border-dashed border-border hover:border-primary/60 bg-muted/30 flex flex-col items-center justify-center cursor-pointer overflow-hidden group transition-all"
                >
                  {photoBase64 ? (
                    <>
                      <img
                        src={photoBase64}
                        alt="Student preview"
                        className="w-full h-full object-cover"
                      />
                      <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity text-white text-xs font-medium">
                        Change Photo
                      </div>
                    </>
                  ) : (
                    <div className="flex flex-col items-center gap-1 p-2 text-center text-muted-foreground">
                      <Camera className="h-6 w-6 text-muted-foreground/60" />
                      <span className="text-[10px] font-medium leading-tight">
                        Upload Photo (Max 20MB)
                      </span>
                    </div>
                  )}
                  {uploadingPhoto && (
                    <div className="absolute inset-0 bg-background/80 flex items-center justify-center">
                      <Loader2 className="h-5 w-5 animate-spin text-primary" />
                    </div>
                  )}
                </div>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/jpeg,image/png,image/webp"
                  className="hidden"
                  onChange={handlePhotoSelect}
                />
              </div>

              {/* Student Fields */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3 flex-1 w-full">
                {useCustomCode && (
                  <div className="md:col-span-2">
                    <label className="text-xs font-semibold text-foreground">
                      Manual Student Code <span className="text-destructive">*</span>
                    </label>
                    <Input
                      className="mt-1 font-mono uppercase"
                      value={customCode}
                      onChange={(e) => setCustomCode(e.target.value)}
                      placeholder="e.g. S2026-0001"
                      required={useCustomCode}
                    />
                  </div>
                )}

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Full Name (English) <span className="text-destructive">*</span>
                  </label>
                  <Input
                    className="mt-1"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Student full legal name"
                    required
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Full Name (বাংলা)
                  </label>
                  <Input
                    className="mt-1 font-bengali"
                    value={nameBn}
                    onChange={(e) => setNameBn(e.target.value)}
                    placeholder="শিক্ষার্থীর পূর্ণ নাম"
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Gender <span className="text-destructive">*</span>
                  </label>
                  <select
                    className="mt-1 flex h-9 w-full rounded-[var(--radius-md)] border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    value={gender}
                    onChange={(e) => setGender(e.target.value)}
                    required
                  >
                    <option value="male">Male (পুরুষ)</option>
                    <option value="female">Female (মহিলা)</option>
                    <option value="other">Other (অন্যান্য)</option>
                  </select>
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Date of Birth <span className="text-destructive">*</span>
                  </label>
                  <Input
                    type="date"
                    className="mt-1"
                    value={dob}
                    onChange={(e) => setDob(e.target.value)}
                    required
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Blood Group
                  </label>
                  <select
                    className="mt-1 flex h-9 w-full rounded-[var(--radius-md)] border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    value={bloodGroup}
                    onChange={(e) => setBloodGroup(e.target.value)}
                  >
                    <option value="">Select Blood Group</option>
                    <option value="A+">A+</option>
                    <option value="A-">A-</option>
                    <option value="B+">B+</option>
                    <option value="B-">B-</option>
                    <option value="O+">O+</option>
                    <option value="O-">O-</option>
                    <option value="AB+">AB+</option>
                    <option value="AB-">AB-</option>
                  </select>
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Religion
                  </label>
                  <select
                    className="mt-1 flex h-9 w-full rounded-[var(--radius-md)] border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    value={religion}
                    onChange={(e) => setReligion(e.target.value)}
                  >
                    <option value="Islam">Islam</option>
                    <option value="Hinduism">Hinduism</option>
                    <option value="Buddhism">Buddhism</option>
                    <option value="Christianity">Christianity</option>
                    <option value="Other">Other</option>
                  </select>
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Student Mobile Phone <span className="text-destructive">*</span>
                  </label>
                  <Input
                    className="mt-1"
                    value={phone}
                    onChange={(e) => setPhone(e.target.value)}
                    placeholder="017XXXXXXXX"
                    required
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Emergency Contact Phone
                  </label>
                  <Input
                    className="mt-1"
                    value={emergencyPhone}
                    onChange={(e) => setEmergencyPhone(e.target.value)}
                    placeholder="Optional emergency number"
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">Email</label>
                  <Input
                    type="email"
                    className="mt-1"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="student@example.com"
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    NID or Birth Cert No.
                  </label>
                  <Input
                    className="mt-1"
                    value={nidBirthCert}
                    onChange={(e) => setNidBirthCert(e.target.value)}
                    placeholder="National ID or Birth Registration"
                  />
                </div>

                <div className="md:col-span-2">
                  <label className="text-xs font-semibold text-foreground">
                    Present Address <span className="text-destructive">*</span>
                  </label>
                  <Input
                    className="mt-1"
                    value={presentAddress}
                    onChange={(e) => setPresentAddress(e.target.value)}
                    placeholder="House, Road, Area, Dhaka"
                    required
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Permanent Address
                  </label>
                  <Input
                    className="mt-1"
                    value={permanentAddress}
                    onChange={(e) => setPermanentAddress(e.target.value)}
                    placeholder="Village, Thana, District"
                  />
                </div>

                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <label className="text-xs font-semibold text-foreground">
                      District
                    </label>
                    <Input
                      className="mt-1"
                      value={district}
                      onChange={(e) => setDistrict(e.target.value)}
                      placeholder="e.g. Dhaka"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-semibold text-foreground">Thana</label>
                    <Input
                      className="mt-1"
                      value={thana}
                      onChange={(e) => setThana(e.target.value)}
                      placeholder="e.g. Dhanmondi"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Section 2: Academic Details */}
          <div className="space-y-4">
            <div className="border-b border-border pb-2">
              <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground">
                2. Academic & Enrollment Details
              </h3>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div>
                <label className="text-xs font-semibold text-foreground">
                  Class / Level
                </label>
                <Input
                  className="mt-1"
                  value={classLevel}
                  onChange={(e) => setClassLevel(e.target.value)}
                  placeholder="e.g. Class 10, HSC-1st"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  School Roll No.
                </label>
                <Input
                  className="mt-1"
                  value={roll}
                  onChange={(e) => setRoll(e.target.value)}
                  placeholder="e.g. 102"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Group / Major
                </label>
                <select
                  className="mt-1 flex h-9 w-full rounded-[var(--radius-md)] border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  value={groupMajor}
                  onChange={(e) => setGroupMajor(e.target.value)}
                >
                  <option value="General">General</option>
                  <option value="Science">Science (বিজ্ঞান)</option>
                  <option value="Commerce">Commerce (ব্যবসায় শিক্ষা)</option>
                  <option value="Humanities">Humanities (মানবিক)</option>
                </select>
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Previous School
                </label>
                <Input
                  className="mt-1"
                  value={previousSchool}
                  onChange={(e) => setPreviousSchool(e.target.value)}
                  placeholder="e.g. Ideal School and College"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Admission Date
                </label>
                <Input
                  type="date"
                  className="mt-1"
                  value={admissionDate}
                  onChange={(e) => setAdmissionDate(e.target.value)}
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Admission Notes
                </label>
                <Input
                  className="mt-1"
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                  placeholder="Special instructions or scholarship"
                />
              </div>
            </div>
          </div>

          {/* Section 3: Primary Guardian & Sibling Link */}
          <div className="space-y-4">
            <div className="flex items-center justify-between border-b border-border pb-2">
              <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground">
                3. Primary Guardian & Sibling Linking
              </h3>
              <div className="flex items-center gap-1 bg-muted p-0.5 rounded-[var(--radius-md)]">
                <button
                  type="button"
                  onClick={() => setGuardianMode("new")}
                  className={`px-3 py-1 text-xs font-semibold rounded-[var(--radius-sm)] transition-all cursor-pointer ${
                    guardianMode === "new"
                      ? "bg-card text-foreground shadow-xs"
                      : "text-muted-foreground hover:text-foreground"
                  }`}
                >
                  New Guardian
                </button>
                <button
                  type="button"
                  onClick={() => setGuardianMode("existing")}
                  className={`px-3 py-1 text-xs font-semibold rounded-[var(--radius-sm)] transition-all cursor-pointer ${
                    guardianMode === "existing"
                      ? "bg-card text-foreground shadow-xs"
                      : "text-muted-foreground hover:text-foreground"
                  }`}
                >
                  Link Sibling's Guardian
                </button>
              </div>
            </div>

            {guardianMode === "existing" ? (
              <div className="space-y-3">
                <div className="relative">
                  <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                  <Input
                    className="pl-9"
                    value={guardianSearchQuery}
                    onChange={(e) => setGuardianSearchQuery(e.target.value)}
                    placeholder="Search guardian by name or phone to link sibling..."
                  />
                  {searchingGuardians && (
                    <Loader2 className="absolute right-3 top-2.5 h-4 w-4 animate-spin text-muted-foreground" />
                  )}
                </div>

                {selectedGuardian ? (
                  <div className="p-3.5 rounded-[var(--radius-md)] bg-primary/10 border border-primary/30 flex items-center justify-between">
                    <div>
                      <div className="flex items-center gap-2">
                        <CheckCircle2 className="h-4 w-4 text-primary" />
                        <span className="text-sm font-bold text-foreground">
                          {selectedGuardian.name}
                        </span>
                        <span className="text-xs capitalize px-2 py-0.5 rounded bg-muted text-muted-foreground">
                          {selectedGuardian.relationship}
                        </span>
                      </div>
                      <div className="text-xs text-muted-foreground mt-0.5">
                        Phone: {selectedGuardian.phone_norm} • Siblings enrolled:{" "}
                        <span className="font-semibold text-primary">
                          {selectedGuardian.student_count}
                        </span>
                      </div>
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => setSelectedGuardian(null)}
                      className="h-8 w-8 p-0"
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                ) : (
                  guardianSearchResults.length > 0 && (
                    <div className="border border-border rounded-[var(--radius-md)] max-h-48 overflow-y-auto divide-y divide-border bg-card">
                      {guardianSearchResults.map((g) => (
                        <div
                          key={g.id}
                          onClick={() => {
                            setSelectedGuardian(g);
                            setGuardianSearchResults([]);
                          }}
                          className="p-2.5 hover:bg-muted/50 cursor-pointer flex items-center justify-between transition-colors"
                        >
                          <div>
                            <div className="text-sm font-medium text-foreground">
                              {g.name} {g.name_bn ? `(${g.name_bn})` : ""}
                            </div>
                            <div className="text-xs text-muted-foreground">
                              {g.phone_norm} • {g.relationship} •{" "}
                              {g.present_address || "No address"}
                            </div>
                          </div>
                          <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-primary/10 text-primary">
                            {g.student_count} student{g.student_count === 1 ? "" : "s"}
                          </span>
                        </div>
                      ))}
                    </div>
                  )
                )}
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Guardian Full Name <span className="text-destructive">*</span>
                  </label>
                  <Input
                    className="mt-1"
                    value={gName}
                    onChange={(e) => setGName(e.target.value)}
                    placeholder="Parent / Legal Guardian name"
                    required={guardianMode === "new"}
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Guardian Name (বাংলা)
                  </label>
                  <Input
                    className="mt-1 font-bengali"
                    value={gNameBn}
                    onChange={(e) => setGNameBn(e.target.value)}
                    placeholder="অভিভাবকের নাম"
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Relationship <span className="text-destructive">*</span>
                  </label>
                  <select
                    className="mt-1 flex h-9 w-full rounded-[var(--radius-md)] border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    value={gRelationship}
                    onChange={(e) => setGRelationship(e.target.value)}
                    required
                  >
                    <option value="father">Father (বাবা)</option>
                    <option value="mother">Mother (মা)</option>
                    <option value="brother">Brother (ভাই)</option>
                    <option value="sister">Sister (বোন)</option>
                    <option value="uncle">Uncle (চাচা / মামা)</option>
                    <option value="legal_guardian">Legal Guardian (আইনি অভিভাবক)</option>
                    <option value="other">Other (অন্যান্য)</option>
                  </select>
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Guardian Mobile Phone <span className="text-destructive">*</span>
                  </label>
                  <Input
                    className="mt-1"
                    value={gPhone}
                    onChange={(e) => setGPhone(e.target.value)}
                    placeholder="017XXXXXXXX"
                    required={guardianMode === "new"}
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Alternative Phone
                  </label>
                  <Input
                    className="mt-1"
                    value={gPhone2}
                    onChange={(e) => setGPhone2(e.target.value)}
                    placeholder="Optional second number"
                  />
                </div>

                <div>
                  <label className="text-xs font-semibold text-foreground">
                    Occupation
                  </label>
                  <Input
                    className="mt-1"
                    value={gOccupation}
                    onChange={(e) => setGOccupation(e.target.value)}
                    placeholder="e.g. Business, Doctor, Teacher"
                  />
                </div>

                <div className="md:col-span-2">
                  <label className="text-xs font-semibold text-foreground">
                    Guardian Address
                  </label>
                  <Input
                    className="mt-1"
                    value={gAddress}
                    onChange={(e) => setGAddress(e.target.value)}
                    placeholder="Leave blank to use student's present address"
                  />
                </div>
              </div>
            )}

            {/* Flags */}
            <div className="flex items-center gap-6 pt-2">
              <label className="flex items-center gap-2 text-xs font-medium text-foreground cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={isPrimary}
                  onChange={(e) => setIsPrimary(e.target.checked)}
                  className="rounded text-primary focus:ring-primary"
                />
                Primary Contact Guardian
              </label>

              <label className="flex items-center gap-2 text-xs font-medium text-foreground cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={receivesSms}
                  onChange={(e) => setReceivesSms(e.target.checked)}
                  className="rounded text-primary focus:ring-primary"
                />
                Receives Attendance & Fee SMS
              </label>
            </div>
          </div>

          <DialogFooter className="pt-4 gap-2 border-t border-border">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={submitting}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={submitting}>
              {submitting ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Admitting Student...
                </>
              ) : (
                "Admit Student"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
