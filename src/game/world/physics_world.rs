use macroquad::time::get_frame_time;
use rapier2d::prelude::*;

pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub gravity: Vector<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl PhysicsWorld {
    pub fn new() -> Self {
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let gravity = vector![0.0, 9.81];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let physics_hooks = ();
        let event_handler = ();

        Self {
            rigid_body_set,
            collider_set,
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            physics_hooks,
            event_handler,
        }
    }

    pub fn update(&mut self) {
        self.integration_parameters.dt = get_frame_time();
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    pub fn add_body(
        &mut self,
        body: RigidBody,
        collider: Collider,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let handle = self.rigid_body_set.insert(body);
        let collider_handle =
            self.collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        (handle, collider_handle)
    }

    pub fn remove_body(&mut self, handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }

    pub fn get_body(&self, handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.rigid_body_set.get(handle)
    }

    pub fn get_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_body_set.get_mut(handle)
    }

    pub fn add_collider(&mut self, collider: Collider) -> ColliderHandle {
        self.collider_set.insert(collider)
    }

    pub fn remove_collider(&mut self, handle: ColliderHandle) {
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            true,
        );
    }

    pub fn get_collider(&self, handle: ColliderHandle) -> Option<&Collider> {
        self.collider_set.get(handle)
    }

    pub fn get_collider_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        self.collider_set.get_mut(handle)
    }
}

impl std::ops::Index<RigidBodyHandle> for PhysicsWorld {
    type Output = RigidBody;

    fn index(&self, index: RigidBodyHandle) -> &Self::Output {
        &self.rigid_body_set[index]
    }
}

impl std::ops::IndexMut<RigidBodyHandle> for PhysicsWorld {
    fn index_mut(&mut self, index: RigidBodyHandle) -> &mut Self::Output {
        &mut self.rigid_body_set[index]
    }
}
